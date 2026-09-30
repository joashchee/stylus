//! stb_image_resize2 by Jeff Roberts and Jorge L Rodriguez
//! (github.com/nothings/stb, public domain or MIT), v2.04, ported from
//! `stb_image_resize2.h` for what `stbir_resize_uint8_linear` does to an
//! RGB image: the default filters (Mitchell shrinking, Catmull-Rom
//! growing, point sampling at 1:1), clamped edges, and the scalar
//! kernels' order of float operations. Its SIMD kernels give the same
//! results on the images checked.
//!
//! Coefficients are f32 and built as the library builds them: gathered
//! per output pixel (or, for a scatter, per input pixel and pivoted, which
//! gives the same values), normalized, copied across repeating phases when
//! the scale is a small rational, and folded back in at clamped edges. A
//! horizontal pass sums up to 3 taps in order, and wider filters in four
//! running sums by tap index (mod 4), added as (a+c)+(b+d); a vertical
//! pass sums rows in order. Which pass runs first follows the library's
//! cost estimate.

use image::RgbaImage;

/// `stbir__small_float`: 2^-120.
const SMALL: f32 = 1.0 / (1u128 << 120) as f32;

#[derive(Clone, Copy, PartialEq)]
enum Filter {
    Mitchell,
    CatmullRom,
    Point,
}

impl Filter {
    fn kernel(self, x: f32) -> f32 {
        let x = x.abs();
        match self {
            Filter::Mitchell => {
                if x < 1.0 {
                    (16.0 + x * x * (21.0 * x - 36.0)) / 18.0
                } else if x < 2.0 {
                    (32.0 + x * (-60.0 + x * (36.0 - 7.0 * x))) / 18.0
                } else {
                    0.0
                }
            }
            Filter::CatmullRom => {
                if x < 1.0 {
                    1.0 - x * x * (2.5 - 1.5 * x)
                } else if x < 2.0 {
                    2.0 - x * (4.0 + x * (0.5 * x - 2.5))
                } else {
                    0.0
                }
            }
            // `stbir__filter_point`.
            Filter::Point => 1.0,
        }
    }

    fn support(self) -> f32 {
        match self {
            Filter::Point => 0.5,
            _ => 2.0,
        }
    }
}

/// One axis's contributors (first and last source pixel) and weights.
struct Axis {
    filter: Filter,
    scale: f32,
    filter_pixel_width: i32,
    gather: bool,
    contribs: Vec<(i32, i32)>,
    coeffs: Vec<Vec<f32>>,
    widest: usize,
}

/// `stbir__double_to_rational`.
fn double_to_rational(f: f64, limit: u32, limit_denom: bool) -> (bool, u32, u32) {
    let (mut top, mut bot) = ((f * (1u64 << 25) as f64) as u64, 1u64 << 25);
    let (mut numer_last, mut denom_last, mut numer_est, mut denom_est) = (0u64, 1u64, 1u64, 0u64);
    loop {
        if (if limit_denom { denom_est } else { numer_est }) >= limit as u64 {
            break;
        }
        if denom_est != 0 {
            let err = (numer_est as f64 / denom_est as f64 - f).abs();
            if err < 1.0 / (1u64 << 24) as f64 {
                return (true, numer_est as u32, denom_est as u32);
            }
        }
        if bot == 0 {
            break;
        }
        let est = top / bot;
        let temp = top % bot;
        top = bot;
        bot = temp;
        let t = est * denom_est + denom_last;
        denom_last = denom_est;
        denom_est = t;
        let t = est * numer_est + numer_last;
        numer_last = numer_est;
        numer_est = t;
    }
    if limit_denom {
        numer_est = (f * limit as f64 + 0.5) as u64;
        denom_est = limit as u64;
    } else {
        numer_est = limit as u64;
        denom_est = (limit as f64 / f + 0.5) as u64;
    }
    let (n, d) = (numer_est as u32, denom_est as u32);
    let err = if d != 0 { (n as f64 / d as f64 - f).abs() } else { 1.0 };
    (err < 1.0 / (1u64 << 24) as f64, n, d)
}

/// `stbir__insert_coeff`.
fn insert_coeff(c: &mut (i32, i32), coeffs: &mut Vec<f32>, new_pixel: i32, new_coeff: f32) {
    if new_pixel <= c.1 {
        if new_pixel < c.0 {
            let o = (c.0 - new_pixel) as usize;
            let len = (c.1 - c.0 + 1) as usize;
            coeffs.resize(coeffs.len().max(len + o), 0.0);
            for j in (0..len).rev() {
                coeffs[j + o] = coeffs[j];
            }
            for v in coeffs.iter_mut().take(o).skip(1) {
                *v = 0.0;
            }
            coeffs[0] = new_coeff;
            c.0 = new_pixel;
        } else {
            coeffs[(new_pixel - c.0) as usize] += new_coeff;
        }
    } else {
        let e = (new_pixel - c.0) as usize;
        coeffs.resize(coeffs.len().max(e + 1), 0.0);
        for v in coeffs.iter_mut().take(e).skip((c.1 - c.0 + 1) as usize) {
            *v = 0.0;
        }
        coeffs[e] = new_coeff;
        c.1 = new_pixel;
    }
}

impl Axis {
    /// `stbir__set_sampler` and `stbir__calculate_filters` for an axis of
    /// `input` pixels resized to `output`, clamped edges.
    fn new(input: usize, output: usize, always_gather: bool) -> Axis {
        let scale_d = output as f64 / input as f64;
        let scale = scale_d as f32;
        let inv_scale = (1.0 / scale_d) as f32;
        let upscale = scale >= 1.0 - SMALL;
        let filter = if upscale {
            if scale <= 1.0 + SMALL {
                Filter::Point
            } else {
                Filter::CatmullRom
            }
        } else {
            Filter::Mitchell
        };
        let support = filter.support();
        let filter_pixel_width = if upscale { (support * 2.0).ceil() as i32 } else { (support * 2.0 / scale).ceil() as i32 };
        let is_gather = if upscale {
            1
        } else if always_gather || filter_pixel_width <= 32 {
            2
        } else {
            0
        };
        let margin = filter_pixel_width / 2;
        let (rational, numerator, denominator) = double_to_rational(scale_d, if scale_d <= 1.0 { output as u32 } else { input as u32 }, scale_d >= 1.0);
        let num = output;
        let polyphase = rational && (numerator as usize) < num;
        let end = if polyphase { numerator as usize } else { num };
        let mut contribs = vec![(0i32, -1i32); num];
        let mut coeffs: Vec<Vec<f32>> = vec![Vec::new(); num];

        if is_gather == 1 {
            // `stbir__calculate_coefficients_for_gather_upsample`.
            let radius = support * scale;
            for n in 0..end {
                let center = n as f32 + 0.5;
                let in_center = center * inv_scale;
                let mut first = ((center - radius) * inv_scale + 0.5).floor() as i32;
                let last = ((center + radius) * inv_scale - 0.5).floor() as i32;
                let mut last_non_zero = -1i32;
                let mut row = Vec::new();
                let mut i = 0i32;
                while i <= last - first {
                    let c = filter.kernel(in_center - ((i + first) as f32 + 0.5));
                    if c < SMALL && c > -SMALL {
                        if i == 0 {
                            first += 1;
                            continue;
                        }
                        row.push(0.0);
                    } else {
                        last_non_zero = i;
                        row.push(c);
                    }
                    i += 1;
                }
                contribs[n] = (first, last_non_zero + first);
                coeffs[n] = row;
            }
        } else {
            // `stbir__calculate_coefficients_for_gather_downsample`.
            let radius = support * inv_scale;
            let mut first_out_inited = -1i32;
            for in_pixel in -margin..(input as i32 + margin) {
                let in_center = in_pixel as f32 + 0.5;
                let out_center = in_center * scale;
                let first = (((in_center - radius) * scale + 0.5).floor() as i32).max(0);
                let mut last = (((in_center + radius) * scale - 0.5).floor() as i32).min(num as i32 - 1);
                if first > last {
                    continue;
                }
                if polyphase {
                    if first == numerator as i32 {
                        break;
                    }
                    if last >= numerator as i32 {
                        last = numerator as i32 - 1;
                    }
                }
                for out in first..=last {
                    let x = (out as f32 + 0.5) - out_center;
                    let mut c = filter.kernel(x) * scale;
                    if c < SMALL && c > -SMALL {
                        c = 0.0;
                    }
                    let (cn, row) = (&mut contribs[out as usize], &mut coeffs[out as usize]);
                    if out > first_out_inited {
                        first_out_inited = out;
                        *cn = (in_pixel, in_pixel);
                        *row = vec![c];
                    } else {
                        if row[0] == 0.0 {
                            cn.0 = in_pixel;
                        }
                        cn.1 = in_pixel;
                        let k = (in_pixel - cn.0) as usize;
                        row.resize(row.len().max(k + 1), 0.0);
                        row[k] = c;
                    }
                }
            }
        }

        // `stbir__cleanup_gathered_coefficients`: normalize.
        for n in 0..end {
            let e = (contribs[n].1 - contribs[n].0) as usize;
            let row = &mut coeffs[n];
            row.resize(row.len().max(e + 1), 0.0);
            let total = row[..=e].iter().fold(0.0f32, |s, &v| s + v);
            if total < SMALL && total > -SMALL {
                contribs[n].1 = contribs[n].0;
                row[0] = 0.0;
            } else if total < 1.0 - SMALL || total > 1.0 + SMALL {
                let fs = 1.0 / total;
                for v in row[..=e].iter_mut() {
                    *v *= fs;
                }
            }
        }
        // The repeating phases, copied.
        if polyphase {
            for n in numerator as usize..num {
                let p = contribs[n - numerator as usize];
                contribs[n] = (p.0 + denominator as i32, p.1 + denominator as i32);
                coeffs[n] = coeffs[n - numerator as usize].clone();
            }
        }
        // Clamped edges, then trailing zeros.
        let last_in = input as i32 - 1;
        let clamp = |i: i32| i.clamp(0, last_in);
        let mut widest = 0usize;
        for n in 0..num {
            let (c, row) = (&mut contribs[n], &mut coeffs[n]);
            if c.1 > last_in {
                let (start, endi) = (c.0, c.1);
                c.1 = last_in;
                for i in input as i32..=endi {
                    let v = row[(i - start) as usize];
                    insert_coeff(c, row, clamp(i), v);
                }
            }
            if c.0 < 0 {
                // Coefficients of pixels -1 down to n0+1, then n0 itself.
                let base = c.0;
                for i in (base + 1..=-1).rev() {
                    let v = row[(i - base) as usize];
                    insert_coeff(c, row, clamp(i), v);
                }
                let save_n0 = c.0;
                let save = row[0];
                c.0 = 0;
                let shift = (-save_n0) as usize;
                if c.1 >= 0 {
                    row.resize(row.len().max(c.1 as usize + 1 + shift), 0.0);
                    for i in 0..=c.1 as usize {
                        row[i] = row[i + shift];
                    }
                }
                insert_coeff(c, row, clamp(save_n0), save);
            }
            if c.0 <= c.1 {
                let mut diff = (c.1 - c.0 + 1) as usize;
                row.resize(row.len().max(diff), 0.0);
                while diff > 0 && row[diff - 1] == 0.0 {
                    diff -= 1;
                }
                c.1 = c.0 + diff as i32 - 1;
                row.truncate(diff);
                if c.0 <= c.1 {
                    widest = widest.max(diff);
                }
            }
        }
        Axis { filter, scale, filter_pixel_width, gather: is_gather != 0, contribs, coeffs, widest }
    }

    /// `stbir__pack_coefficients`'s shift at the right edge: a filter that
    /// would read past `row_width` starts earlier, with zero weights first
    /// (which moves taps to other running sums).
    fn pack(&mut self, row_width: i32) {
        let widest = self.widest as i32;
        let stop_for = |c: (i32, i32)| {
            if widest > 12 {
                let m = widest & 3;
                ((((c.1 - c.0 + 1) - m + 3) & !3) + m).max(8 + m)
            } else {
                widest
            }
        };
        for n in (0..self.contribs.len()).rev() {
            let c = self.contribs[n];
            if c.0 + widest * 2 < row_width {
                break;
            }
            if c.0 + widest > row_width {
                let stop = stop_for(c);
                if c.0 + stop > row_width {
                    let new_n0 = row_width - stop;
                    let backup = (c.0 - new_n0) as usize;
                    let row = &mut self.coeffs[n];
                    let mut shifted = vec![0.0f32; backup];
                    shifted.extend_from_slice(row);
                    *row = shifted;
                    self.contribs[n].0 = new_n0;
                }
            }
        }
    }

    /// The horizontal kernels on one decoded row of `ch`-channel pixels.
    fn horizontal(&self, src: &[f32], out: &mut [f32]) {
        for (o, (&(n0, _), row)) in self.contribs.iter().zip(&self.coeffs).enumerate() {
            for c in 0..3 {
                let tap = |k: usize| {
                    let w = row.get(k).copied().unwrap_or(0.0);
                    src[(n0 as usize + k) * 3 + c] * w
                };
                out[o * 3 + c] = if self.widest <= 3 {
                    let mut t = tap(0);
                    for k in 1..self.widest {
                        t += tap(k);
                    }
                    t
                } else {
                    let mut acc = [0f32; 4];
                    // Taps up to the widest, or the n-coeff loops' reach.
                    let count = row.len().max(4);
                    for k in 0..count {
                        acc[k % 4] = if k < 4 { tap(k) } else { acc[k % 4] + tap(k) };
                    }
                    (acc[0] + acc[2]) + (acc[1] + acc[3])
                };
            }
        }
    }
}

/// `stbir__should_do_vertical_first`, with the 3-channel weights.
fn vertical_first(h: &Axis, h_out: usize, v: &Axis, v_out: usize) -> bool {
    const WEIGHTS: [[f32; 4]; 8] = [
        [0.00000, 0.53125, 0.00000, 0.03125],
        [0.06250, 0.96875, 0.00000, 0.53125],
        [0.87500, 0.18750, 0.00000, 0.93750],
        [0.00000, 0.09375, 1.00000, 1.00000],
        [1.00000, 1.00000, 1.00000, 1.00000],
        [0.03125, 0.12500, 1.00000, 1.00000],
        [0.06250, 0.12500, 0.00000, 1.00000],
        [0.00000, 1.00000, 0.00000, 0.56250],
    ];
    let class = if v_out <= 4 || h_out <= 4 {
        if v_out < h_out {
            6
        } else {
            7
        }
    } else if v.scale <= 1.0 {
        if v.gather {
            1
        } else {
            0
        }
    } else if v.scale <= 2.0 {
        2
    } else if v.scale <= 3.0 {
        3
    } else if v.scale <= 4.0 {
        5
    } else {
        6
    };
    let w = WEIGHTS[class];
    let (hf, vf) = (h.filter_pixel_width as f32, v.filter_pixel_width as f32);
    let h_cost = hf * w[0] + h.scale * vf * w[1];
    let v_cost = vf * w[2] + v.scale * hf * w[3];
    v_cost <= h_cost
}

/// `stbir_resize_uint8_linear` of an RGB image (alpha is ignored).
pub fn resize_rgb(img: &RgbaImage, width: u32, height: u32) -> Vec<[u8; 3]> {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let (nw, nh) = (width.max(1) as usize, height.max(1) as usize);
    // Unscaled decode: 8-bit values as floats (8-bit in, 8-bit out).
    let decode = |y: usize| -> Vec<f32> { (0..w).flat_map(|x| { let p = img.get_pixel(x as u32, y as u32); [p[0] as f32, p[1] as f32, p[2] as f32] }).collect() };
    let mut hz = Axis::new(w, nw, true);
    let vt = Axis::new(h, nh, false);
    hz.pack(w as i32);
    let horizontal = |row: &[f32]| {
        let mut out = vec![0f32; nw * 3];
        if hz.filter == Filter::Point && hz.scale == 1.0 {
            out.copy_from_slice(row);
        } else {
            hz.horizontal(row, &mut out);
        }
        out
    };
    // A vertical output row: its source rows summed in order.
    let vertical = |o: usize, rows: &dyn Fn(usize) -> Vec<f32>| {
        let (n0, _) = vt.contribs[o];
        let weights = &vt.coeffs[o];
        let mut acc: Vec<f32> = Vec::new();
        for (k, &c) in weights.iter().enumerate() {
            let r = rows(n0 as usize + k);
            if k == 0 {
                acc = r.iter().map(|&v| v * c).collect();
            } else {
                acc.iter_mut().zip(&r).for_each(|(a, &v)| *a += v * c);
            }
        }
        if acc.is_empty() {
            acc = vec![0.0; rows(n0 as usize).len()];
        }
        acc
    };
    let rows: Vec<Vec<f32>> = if vertical_first(&hz, nw, &vt, nh) {
        let decoded: Vec<Vec<f32>> = (0..h).map(decode).collect();
        (0..nh).map(|o| horizontal(&vertical(o, &|y| decoded[y].clone()))).collect()
    } else {
        let resized: Vec<Vec<f32>> = (0..h).map(|y| horizontal(&decode(y))).collect();
        (0..nh).map(|o| vertical(o, &|y| resized[y].clone())).collect()
    };
    // `stbir__encode_uint8_linear`: + 0.5, clamped, truncated.
    rows.iter()
        .flat_map(|r| r.chunks(3).map(|p| p.iter().map(|&f| (f + 0.5).clamp(0.0, 255.0) as u8).collect::<Vec<u8>>().try_into().unwrap()).collect::<Vec<[u8; 3]>>())
        .collect()
}
