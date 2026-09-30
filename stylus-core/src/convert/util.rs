//! Helpers the converter ports share: the resampling filters the original
//! tools get from their image libraries, the standard terminal palettes,
//! color math and a seeded PRNG.

use image::{Rgba, RgbaImage};

/// A resampling kernel, as Pillow, Go's nfnt/resize and `imaging`, and a
/// browser canvas scale images: a separable convolution whose support
/// widens by the scale factor when shrinking, so every source pixel counts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kernel {
    /// Area average (Pillow's BOX).
    Box,
    /// Linear (Pillow's BILINEAR, nfnt's Bilinear, a canvas's smoothing).
    Triangle,
    /// Keys cubic with a = -0.5 (Pillow's BICUBIC, nfnt's Bicubic,
    /// Catmull-Rom).
    Bicubic,
    /// Lanczos with 3 lobes (Pillow's LANCZOS/ANTIALIAS, nfnt's Lanczos3,
    /// `imaging.Lanczos`).
    Lanczos3,
}

impl Kernel {
    fn support(self) -> f64 {
        match self {
            Kernel::Box => 0.5,
            Kernel::Triangle => 1.0,
            Kernel::Bicubic => 2.0,
            Kernel::Lanczos3 => 3.0,
        }
    }

    fn weight(self, signed: f64) -> f64 {
        let x = signed.abs();
        match self {
            Kernel::Box => {
                // Pillow's box: -0.5 < x <= 0.5 (before taking |x|).
                if x < 0.5 || (x == 0.5 && signed > 0.0) {
                    1.0
                } else {
                    0.0
                }
            }
            Kernel::Triangle => (1.0 - x).max(0.0),
            Kernel::Bicubic => {
                let a = -0.5;
                if x < 1.0 {
                    ((a + 2.0) * x - (a + 3.0)) * x * x + 1.0
                } else if x < 2.0 {
                    (((x - 5.0) * x + 8.0) * x - 4.0) * a
                } else {
                    0.0
                }
            }
            Kernel::Lanczos3 => {
                if x < 3.0 {
                    sinc(x) * sinc(x / 3.0)
                } else {
                    0.0
                }
            }
        }
    }
}

fn sinc(x: f64) -> f64 {
    if x == 0.0 {
        1.0
    } else {
        let px = std::f64::consts::PI * x;
        px.sin() / px
    }
}

/// Per output pixel: the first source pixel and the normalized weights,
/// as Pillow's `precompute_coeffs` computes them.
fn coefficients(in_size: u32, out_size: u32, kernel: Kernel) -> Vec<(usize, Vec<f64>)> {
    let scale = in_size as f64 / out_size as f64;
    let filter_scale = scale.max(1.0);
    let support = kernel.support() * filter_scale;
    (0..out_size)
        .map(|i| {
            let center = (i as f64 + 0.5) * scale;
            // C's (int) cast truncates toward zero.
            let min = ((center - support + 0.5) as i64).max(0) as usize;
            let max = ((center + support + 0.5) as i64).min(in_size as i64) as usize;
            let mut weights: Vec<f64> = (min..max).map(|x| kernel.weight((x as f64 - center + 0.5) / filter_scale)).collect();
            let total: f64 = weights.iter().sum();
            if total != 0.0 {
                weights.iter_mut().for_each(|w| *w /= total);
            }
            (min, weights)
        })
        .collect()
}

/// Pillow's fixed-point precision for 8-bit resampling.
const PRECISION_BITS: u32 = 32 - 8 - 2;

/// Weights in Pillow's fixed point, rounded half away from zero.
fn fixed(coefficients: Vec<(usize, Vec<f64>)>) -> Vec<(usize, Vec<i64>)> {
    let one = (1i64 << PRECISION_BITS) as f64;
    coefficients
        .into_iter()
        .map(|(start, ws)| (start, ws.into_iter().map(|w| if w < 0.0 { (w * one - 0.5) as i64 } else { (w * one + 0.5) as i64 }).collect()))
        .collect()
}

/// Pillow's `clip8`: a fixed-point sum (with the rounding half already
/// added) back to 0–255.
fn clip8(sum: i64) -> u8 {
    (sum >> PRECISION_BITS).clamp(0, 255) as u8
}

/// One resampling pass over rows (`horizontal`) or columns, on 4-channel
/// pixels, as Pillow's `ImagingResampleHorizontal_8bpc` and `..Vertical_`.
fn pass(src: &[[u8; 4]], w: usize, h: usize, out_len: usize, horizontal: bool, coeffs: &[(usize, Vec<i64>)]) -> Vec<[u8; 4]> {
    let (ow, oh) = if horizontal { (out_len, h) } else { (w, out_len) };
    let mut out = vec![[0u8; 4]; ow * oh];
    for y in 0..oh {
        for x in 0..ow {
            let (start, ks) = &coeffs[if horizontal { x } else { y }];
            let mut acc = [1i64 << (PRECISION_BITS - 1); 4];
            for (k, &kk) in ks.iter().enumerate() {
                let p = if horizontal { src[y * w + start + k] } else { src[(start + k) * w + x] };
                for c in 0..4 {
                    acc[c] += p[c] as i64 * kk;
                }
            }
            out[y * ow + x] = acc.map(clip8);
        }
    }
    out
}

/// Resizes exactly as Pillow's `Image.resize` does: RGBA is premultiplied
/// ("RGBa"), then a horizontal pass and a vertical pass in 8-bit fixed
/// point (each skipped when that size doesn't change), then un-premultiplied.
pub fn resample(img: &RgbaImage, width: u32, height: u32, kernel: Kernel) -> RgbaImage {
    let width = width.max(1);
    let height = height.max(1);
    let (w, h) = img.dimensions();
    if (width, height) == (w, h) {
        return img.clone(); // Pillow returns a copy
    }
    // Pillow's rgba2rgbA: MULDIV255 rounding.
    let muldiv255 = |a: u8, b: u8| {
        let t = a as u32 * b as u32 + 128;
        (((t >> 8) + t) >> 8) as u8
    };
    let mut px: Vec<[u8; 4]> = img.pixels().map(|p| [muldiv255(p[0], p[3]), muldiv255(p[1], p[3]), muldiv255(p[2], p[3]), p[3]]).collect();
    let mut cur_w = w as usize;
    if width != w {
        px = pass(&px, cur_w, h as usize, width as usize, true, &fixed(coefficients(w, width, kernel)));
        cur_w = width as usize;
    }
    if height != h {
        px = pass(&px, cur_w, h as usize, height as usize, false, &fixed(coefficients(h, height, kernel)));
    }
    // Pillow's rgbA2rgba: integer division, clipped.
    let un = |c: u8, a: u8| if a == 0 || a == 255 { c } else { ((255 * c as u32) / a as u32).min(255) as u8 };
    RgbaImage::from_fn(width, height, |x, y| {
        let p = px[(y * width + x) as usize];
        Rgba([un(p[0], p[3]), un(p[1], p[3]), un(p[2], p[3]), p[3]])
    })
}

/// Pillow's `Image.resize` with NEAREST (`ImagingScaleAffine`): each
/// output pixel copies the source pixel under its center, the position
/// stepped by repeated addition as Pillow does.
pub fn pillow_nearest(img: &RgbaImage, width: u32, height: u32) -> RgbaImage {
    let (w, h) = img.dimensions();
    if (width, height) == (w, h) {
        return img.clone();
    }
    let table = |src: u32, dst: u32| {
        let a = src as f64 / dst as f64;
        let mut o = a * 0.5;
        (0..dst)
            .map(|_| {
                let i = if o < 0.0 { 0 } else { (o as u32).min(src - 1) };
                o += a;
                i
            })
            .collect::<Vec<u32>>()
    };
    let (xs, ys) = (table(w, width), table(h, height));
    RgbaImage::from_fn(width, height, |x, y| *img.get_pixel(xs[x as usize], ys[y as usize]))
}

/// Pillow's `alpha_composite` of `src` over an opaque color
/// (`ImagingAlphaComposite`, 7 extra bits of precision), RGB only.
pub fn pillow_over(src: [u8; 4], dst: [u8; 3]) -> [u8; 3] {
    let a = src[3] as u32;
    if a == 0 {
        return dst;
    }
    const PRECISION_BITS: u32 = 7;
    let blend = 255 * (255 - a);
    let outa255 = a * 255 + blend;
    let coef1 = a * 255 * 255 * (1 << PRECISION_BITS) / outa255;
    let coef2 = 255 * (1 << PRECISION_BITS) - coef1;
    let div255 = |t: u32| ((t >> 8) + t) >> 8;
    std::array::from_fn(|c| (div255(src[c] as u32 * coef1 + dst[c] as u32 * coef2 + (0x80 << PRECISION_BITS)) >> PRECISION_BITS) as u8)
}

/// `imaging.Resize(img, w, h, imaging.Lanczos)` from
/// github.com/disintegration/imaging (MIT, Grigory Dryapak), which the Go
/// converters that use it resize with: alpha-weighted separable passes
/// (horizontal, then vertical), each rounded to 8 bits, straight alpha.
pub fn imaging_lanczos(img: &RgbaImage, width: u32, height: u32) -> RgbaImage {
    fn kernel(x: f64) -> f64 {
        let sinc = |x: f64| if x == 0.0 { 1.0 } else { go_sin(std::f64::consts::PI * x) / (std::f64::consts::PI * x) };
        let x = x.abs();
        if x < 3.0 {
            sinc(x) * sinc(x / 3.0)
        } else {
            0.0
        }
    }
    fn weights(dst: usize, src: usize) -> Vec<Vec<(usize, f64)>> {
        let du = src as f64 / dst as f64;
        let scale = du.max(1.0);
        let ru = (scale * 3.0).ceil();
        (0..dst)
            .map(|v| {
                let fu = (v as f64 + 0.5) * du - 0.5;
                let begin = ((fu - ru).ceil() as i64).max(0);
                let end = ((fu + ru).floor() as i64).min(src as i64 - 1);
                let mut ws: Vec<(usize, f64)> = (begin..=end).map(|u| (u as usize, kernel((u as f64 - fu) / scale))).filter(|&(_, w)| w != 0.0).collect();
                let sum: f64 = ws.iter().map(|w| w.1).sum();
                if sum != 0.0 {
                    ws.iter_mut().for_each(|w| w.1 /= sum);
                }
                ws
            })
            .collect()
    }
    fn clamp(x: f64) -> u8 {
        let v = (x + 0.5) as i64;
        v.clamp(0, 255) as u8
    }
    // One pass over lines of `len` pixels, `n` of them, reading with `at`.
    fn pass(n_lines: usize, out_len: usize, src_len: usize, at: &dyn Fn(usize, usize) -> [u8; 4]) -> Vec<Vec<[u8; 4]>> {
        let ws = weights(out_len, src_len);
        (0..n_lines)
            .map(|line| {
                ws.iter()
                    .map(|w| {
                        let (mut r, mut g, mut b, mut a) = (0.0, 0.0, 0.0, 0.0);
                        for &(i, wt) in w {
                            let s = at(line, i);
                            let aw = s[3] as f64 * wt;
                            r += s[0] as f64 * aw;
                            g += s[1] as f64 * aw;
                            b += s[2] as f64 * aw;
                            a += aw;
                        }
                        if a != 0.0 {
                            let inv = 1.0 / a;
                            [clamp(r * inv), clamp(g * inv), clamp(b * inv), clamp(a)]
                        } else {
                            [0; 4]
                        }
                    })
                    .collect()
            })
            .collect()
    }
    let (sw, sh) = (img.width() as usize, img.height() as usize);
    let (dw, dh) = (width.max(1) as usize, height.max(1) as usize);
    let mut cur: Vec<Vec<[u8; 4]>> = (0..sh).map(|y| (0..sw).map(|x| img.get_pixel(x as u32, y as u32).0).collect()).collect();
    if dw != sw {
        let src = cur;
        cur = pass(sh, dw, sw, &|y, x| src[y][x]);
    }
    if dh != sh {
        let src = cur;
        let cols = pass(dw, dh, sh, &|x, y| src[y][x]);
        cur = (0..dh).map(|y| (0..dw).map(|x| cols[x][y]).collect()).collect();
    }
    RgbaImage::from_fn(dw as u32, dh as u32, |x, y| Rgba(cur[y as usize][x as usize]))
}

/// Go's `math.Sin` (Cephes, in pure Go on every platform but s390x), for
/// the Go converters' resamplers: it can differ from the system's `sin` in
/// the last bit, which can tip a rounding. Only the path for |x| < 2^29.
#[allow(clippy::excessive_precision)]
pub fn go_sin(x: f64) -> f64 {
    const SIN: [f64; 6] = [1.58962301576546568060e-10, -2.50507477628578072866e-8, 2.75573136213857245213e-6, -1.98412698295895385996e-4, 8.33333333332211858878e-3, -1.66666666666666307295e-1];
    const COS: [f64; 6] = [-1.13585365213876817300e-11, 2.08757008419747316778e-9, -2.75573141792967388112e-7, 2.48015872888517045348e-5, -1.38888888888730564116e-3, 4.16666666666665929218e-2];
    const PI4A: f64 = 7.85398125648498535156e-1;
    const PI4B: f64 = 3.77489470793079817668e-8;
    const PI4C: f64 = 2.69515142907905952645e-15;
    // Go's constant 4/Pi, rounded once.
    const FOUR_OVER_PI: f64 = 1.2732395447351628;
    if x == 0.0 || x.is_nan() {
        return x;
    }
    if x.is_infinite() || x.abs() >= (1u64 << 29) as f64 {
        return x.sin();
    }
    let (mut x, mut sign) = (x, false);
    if x < 0.0 {
        x = -x;
        sign = true;
    }
    let mut j = (x * FOUR_OVER_PI) as u64;
    let mut y = j as f64;
    if j & 1 == 1 {
        j += 1;
        y += 1.0;
    }
    j &= 7;
    let z = ((x - y * PI4A) - y * PI4B) - y * PI4C;
    if j > 3 {
        sign = !sign;
        j -= 4;
    }
    let zz = z * z;
    let y = if j == 1 || j == 2 {
        1.0 - 0.5 * zz + zz * zz * (((((COS[0] * zz + COS[1]) * zz + COS[2]) * zz + COS[3]) * zz + COS[4]) * zz + COS[5])
    } else {
        z + z * zz * (((((SIN[0] * zz + SIN[1]) * zz + SIN[2]) * zz + SIN[3]) * zz + SIN[4]) * zz + SIN[5])
    };
    if sign {
        -y
    } else {
        y
    }
}

/// The image with alpha ignored (set opaque), keeping each pixel's stored
/// color: what Pillow's `convert("RGB")` and Go's `color.RGBA` reads of an
/// NRGBA pixel's RGB do with transparency.
pub fn drop_alpha(img: &RgbaImage) -> RgbaImage {
    RgbaImage::from_fn(img.width(), img.height(), |x, y| {
        let p = img.get_pixel(x, y);
        Rgba([p[0], p[1], p[2], 255])
    })
}

/// The six levels of xterm's 6×6×6 color cube.
pub const CUBE_LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// xterm's 256 colors, with xterm's own 16 system colors.
pub fn xterm256(index: u8) -> [u8; 3] {
    const SYSTEM: [[u8; 3]; 16] = [
        [0, 0, 0],
        [205, 0, 0],
        [0, 205, 0],
        [205, 205, 0],
        [0, 0, 238],
        [205, 0, 205],
        [0, 205, 205],
        [229, 229, 229],
        [127, 127, 127],
        [255, 0, 0],
        [0, 255, 0],
        [255, 255, 0],
        [92, 92, 255],
        [255, 0, 255],
        [0, 255, 255],
        [255, 255, 255],
    ];
    match index {
        0..=15 => SYSTEM[index as usize],
        16..=231 => {
            let i = index - 16;
            [CUBE_LEVELS[(i / 36) as usize], CUBE_LEVELS[(i / 6 % 6) as usize], CUBE_LEVELS[(i % 6) as usize]]
        }
        _ => {
            let v = 8 + 10 * (index - 232);
            [v, v, v]
        }
    }
}

/// The 16 VGA text-mode colors in SGR order (black, red, green, brown,
/// blue, magenta, cyan, light gray, then the bright ones).
pub const VGA_SGR: [[u8; 3]; 16] = [
    [0, 0, 0],
    [170, 0, 0],
    [0, 170, 0],
    [170, 85, 0],
    [0, 0, 170],
    [170, 0, 170],
    [0, 170, 170],
    [170, 170, 170],
    [85, 85, 85],
    [255, 85, 85],
    [85, 255, 85],
    [255, 255, 85],
    [85, 85, 255],
    [255, 85, 255],
    [85, 255, 255],
    [255, 255, 255],
];

/// The Windows console / "HTML" 16 colors (128 and 192 steps) in SGR
/// order, which several converters use as the system palette.
pub const WINDOWS_SGR: [[u8; 3]; 16] = [
    [0, 0, 0],
    [128, 0, 0],
    [0, 128, 0],
    [128, 128, 0],
    [0, 0, 128],
    [128, 0, 128],
    [0, 128, 128],
    [192, 192, 192],
    [128, 128, 128],
    [255, 0, 0],
    [0, 255, 0],
    [255, 255, 0],
    [0, 0, 255],
    [255, 0, 255],
    [0, 255, 255],
    [255, 255, 255],
];

/// The 256-color palette several Go converters share: the Windows 16
/// system colors, then xterm's cube and grays.
pub fn windows_xterm256(index: u8) -> [u8; 3] {
    if index < 16 {
        WINDOWS_SGR[index as usize]
    } else {
        xterm256(index)
    }
}

/// Go's `color.Palette.Index` over a 256-color palette (all opaque, e.g.
/// `windows_xterm256`), for a color's premultiplied 16-bit channels
/// (`Color.RGBA()`), with Go's uint32 wrapping arithmetic.
pub fn go_palette256_index(palette: fn(u8) -> [u8; 3], c: [u32; 4]) -> u8 {
    let sq_diff = |x: u32, y: u32| {
        let d = x.wrapping_sub(y);
        d.wrapping_mul(d) >> 2
    };
    let mut best = 0;
    let mut best_sum = u32::MAX;
    for i in 0..=255u8 {
        let v = palette(i).map(|v| v as u32 * 0x101);
        let sum = sq_diff(c[0], v[0]).wrapping_add(sq_diff(c[1], v[1])).wrapping_add(sq_diff(c[2], v[2])).wrapping_add(sq_diff(c[3], 0xffff));
        if sum < best_sum {
            if sum == 0 {
                return i;
            }
            best = i;
            best_sum = sum;
        }
    }
    best
}

/// SplitMix64, for the converters whose original picks characters at
/// random: seeded, so a conversion is the same every time.
pub struct SplitMix64(u64);

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        SplitMix64(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A value in `0..n`.
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
}

/// Python's float floor division (`a // b`), which can differ from
/// `(a / b).floor()`: `512 // 6.4` is 79.0, not 80.
pub fn py_floordiv(a: f64, b: f64) -> f64 {
    let m = a % b;
    let mut div = (a - m) / b;
    if m != 0.0 && ((b < 0.0) != (m < 0.0)) {
        div -= 1.0;
    }
    let mut floor = div.floor();
    if div - floor > 0.5 {
        floor += 1.0;
    }
    floor
}

/// Rows for `columns` columns of an image `w`×`h` in cells `cell_aspect`
/// (width ÷ height) wide: the usual terminal-converter sizing.
pub fn rows_for(w: u32, h: u32, columns: u32, cell_aspect: f64) -> u32 {
    ((columns as f64 * h as f64 / w as f64 * cell_aspect).round() as u32).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resampling_a_flat_image_keeps_its_color() {
        let img = RgbaImage::from_pixel(37, 23, Rgba([10, 200, 90, 255]));
        for k in [Kernel::Box, Kernel::Triangle, Kernel::Bicubic, Kernel::Lanczos3] {
            let out = resample(&img, 11, 7, k);
            assert!(out.pixels().all(|p| *p == Rgba([10, 200, 90, 255])), "{k:?}");
            let out = resample(&img, 80, 50, k);
            assert!(out.pixels().all(|p| *p == Rgba([10, 200, 90, 255])), "{k:?} up");
        }
    }

    #[test]
    fn box_filter_averages_areas() {
        let mut img = RgbaImage::new(2, 1);
        img.put_pixel(0, 0, Rgba([0, 0, 0, 255]));
        img.put_pixel(1, 0, Rgba([200, 100, 50, 255]));
        assert_eq!(*resample(&img, 1, 1, Kernel::Box).get_pixel(0, 0), Rgba([100, 50, 25, 255]));
    }

    #[test]
    fn python_floor_division() {
        assert_eq!(py_floordiv(512.0, 6.4), 79.0);
        assert_eq!(py_floordiv(10.0, 2.5), 4.0);
        assert_eq!(py_floordiv(7.0, 2.0), 3.0);
    }

    #[test]
    fn go_sin_is_sin() {
        for i in -2000..2000 {
            let x = i as f64 * 0.01234;
            assert!((go_sin(x) - x.sin()).abs() < 1e-15, "{x}");
        }
    }

    #[test]
    fn xterm_cube_and_grays() {
        assert_eq!(xterm256(16), [0, 0, 0]);
        assert_eq!(xterm256(196), [255, 0, 0]);
        assert_eq!(xterm256(231), [255, 255, 255]);
        assert_eq!(xterm256(232), [8, 8, 8]);
        assert_eq!(xterm256(255), [238, 238, 238]);
    }

}
