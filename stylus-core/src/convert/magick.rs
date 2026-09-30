//! ImageMagick's `ResizeImage` (github.com/ImageMagick/ImageMagick,
//! ImageMagick License), as a Q16 HDRI build runs it, ported from
//! `MagickCore/resize.c` (`AcquireResizeFilter`, `GetResizeFilterWeight`,
//! `SincFast`, `CubicBC`, `HorizontalFilter`, `VerticalFilter`) for the
//! converters that resize through it: an undefined filter picks Lanczos,
//! or Mitchell for an image with alpha or one that grows. Pixels are
//! 16-bit quantums kept as f32 between the passes (HDRI), and the weights
//! and sums are f64.

use image::RgbaImage;

/// `MagickEpsilon`.
const EPSILON: f64 = 1.0e-12;
/// `QuantumScale`.
const QUANTUM_SCALE: f64 = 1.0 / 65535.0;

/// `MagickSafeReciprocal`.
fn safe_reciprocal(x: f64) -> f64 {
    if x > -EPSILON && x < EPSILON {
        1.0 / EPSILON
    } else {
        1.0 / x
    }
}

/// `SincFast` for a Q16 build: the Robidoux–Racette polynomial on
/// [-4, 4], sin(πx)/(πx) beyond. The constants are ImageMagick's, as
/// written.
#[allow(clippy::excessive_precision)]
fn sinc_fast(x: f64) -> f64 {
    if x > 4.0 {
        let alpha = std::f64::consts::PI * x;
        return alpha.sin() / alpha;
    }
    let xx = x * x;
    const C: [f64; 10] = [
        0.173611107357320220183368594093166520811e-2,
        -0.384240921114946632192116762889211361285e-3,
        0.394201182359318128221229891724947048771e-4,
        -0.250963301609117217660068889165550534856e-5,
        0.111902032818095784414237782071368805120e-6,
        -0.372895101408779549368465614321137048875e-8,
        0.957694196677572570319816780188718518330e-10,
        -0.187208577776590710853865174371617338991e-11,
        0.253524321426864752676094495396308636823e-13,
        -0.177084805010701112639035485248501049364e-15,
    ];
    let p = C[0] + xx * (C[1] + xx * (C[2] + xx * (C[3] + xx * (C[4] + xx * (C[5] + xx * (C[6] + xx * (C[7] + xx * (C[8] + xx * C[9]))))))));
    (xx - 1.0) * (xx - 4.0) * (xx - 9.0) * (xx - 16.0) * p
}

#[derive(Clone, Copy, PartialEq)]
enum Filter {
    /// Lanczos: SincFast windowed by SincFast, 3 lobes.
    Lanczos,
    /// Mitchell: CubicBC with B = C = 1/3, support 2, no window.
    Mitchell,
}

impl Filter {
    fn support(self) -> f64 {
        match self {
            Filter::Lanczos => 3.0,
            Filter::Mitchell => 2.0,
        }
    }

    /// `GetResizeFilterWeight` (blur 1).
    fn weight(self, x: f64) -> f64 {
        let x_blur = x.abs() * safe_reciprocal(1.0);
        match self {
            Filter::Lanczos => {
                // The window's scale: its first zero (1) over the support.
                let scale = safe_reciprocal(3.0);
                sinc_fast(x_blur * scale) * sinc_fast(x_blur)
            }
            Filter::Mitchell => {
                let (b, c) = (1.0 / 3.0, 1.0 / 3.0);
                let two_b = b + b;
                let k = [1.0 - (1.0 / 3.0) * b, -3.0 + two_b + c, 2.0 - 1.5 * b - c, (4.0 / 3.0) * b + 4.0 * c, -8.0 * c - two_b, b + 5.0 * c, (-1.0 / 6.0) * b - c];
                if x_blur < 1.0 {
                    k[0] + x_blur * (x_blur * (k[1] + x_blur * k[2]))
                } else if x_blur < 2.0 {
                    k[3] + x_blur * (k[4] + x_blur * (k[5] + x_blur * k[6]))
                } else {
                    0.0
                }
            }
        }
    }
}

/// One pass along an axis (`HorizontalFilter` / `VerticalFilter`): `lines`
/// lines of `len` pixels, read with `at(line, i)`, to `n` pixels each.
fn pass(filter: Filter, lines: usize, len: usize, n: usize, factor: f64, alpha: bool, at: &dyn Fn(usize, usize) -> [f32; 4]) -> Vec<Vec<[f32; 4]>> {
    let mut scale = (1.0 / factor + EPSILON).max(1.0);
    let mut support = scale * filter.support();
    if support < 0.5 {
        support = 0.5;
        scale = 1.0;
    }
    let scale = safe_reciprocal(scale);
    let contributions: Vec<(usize, Vec<f64>)> = (0..n)
        .map(|o| {
            let bisect = (o as f64 + 0.5) / factor + EPSILON;
            let start = (bisect - support + 0.5).max(0.0) as usize;
            let stop = (bisect + support + 0.5).min(len as f64) as usize;
            let mut w: Vec<f64> = (start..stop.max(start)).map(|i| filter.weight(scale * (i as f64 - bisect + 0.5))).collect();
            let density: f64 = w.iter().fold(0.0, |s, &v| s + v);
            if density != 0.0 && density != 1.0 {
                let d = safe_reciprocal(density);
                w.iter_mut().for_each(|v| *v *= d);
            }
            (start, w)
        })
        .collect();
    (0..lines)
        .map(|line| {
            contributions
                .iter()
                .map(|(start, w)| {
                    let mut out = [0f32; 4];
                    // The alpha channel, and every channel of an image
                    // without alpha: a plain weighted sum.
                    for ch in 0..4 {
                        if alpha && ch < 3 {
                            continue;
                        }
                        let mut pixel = 0.0f64;
                        for (j, &wt) in w.iter().enumerate() {
                            pixel += wt * at(line, start + j)[ch] as f64;
                        }
                        out[ch] = pixel as f32;
                    }
                    if alpha {
                        // Color channels weighted by alpha (BlendPixelTrait).
                        for ch in 0..3 {
                            let (mut pixel, mut gamma) = (0.0f64, 0.0f64);
                            for (j, &wt) in w.iter().enumerate() {
                                let p = at(line, start + j);
                                let a = wt * QUANTUM_SCALE * p[3] as f64;
                                pixel += a * p[ch] as f64;
                                gamma += a;
                            }
                            out[ch] = (safe_reciprocal(gamma) * pixel) as f32;
                        }
                    }
                    out
                })
                .collect()
        })
        .collect()
}

/// `ResizeImage(image, width, height, UndefinedFilter)` on an 8-bit image
/// (`alpha`: whether it has an alpha channel), then each channel back to 8
/// bits as `ScaleQuantumToChar` does.
pub fn resize(img: &RgbaImage, width: u32, height: u32, alpha: bool) -> RgbaImage {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let (nw, nh) = (width.max(1) as usize, height.max(1) as usize);
    let src: Vec<[f32; 4]> = img.pixels().map(|p| p.0.map(|v| 257.0 * v as f32)).collect();
    let x_factor = nw as f64 * safe_reciprocal(w as f64);
    let y_factor = nh as f64 * safe_reciprocal(h as f64);
    let out: Vec<[f32; 4]> = if (nw, nh) == (w, h) {
        src
    } else {
        let filter = if alpha || x_factor * y_factor > 1.0 { Filter::Mitchell } else { Filter::Lanczos };
        if x_factor > y_factor {
            let tmp = pass(filter, h, w, nw, x_factor, alpha, &|y, x| src[y * w + x]);
            let cols = pass(filter, nw, h, nh, y_factor, alpha, &|x, y| tmp[y][x]);
            (0..nh * nw).map(|i| cols[i % nw][i / nw]).collect()
        } else {
            let cols = pass(filter, w, h, nh, y_factor, alpha, &|x, y| src[y * w + x]);
            let rows = pass(filter, nh, w, nw, x_factor, alpha, &|y, x| cols[x][y]);
            rows.into_iter().flatten().collect()
        }
    };
    // ScaleQuantumToChar, HDRI: in f32, rounded half up.
    let to_char = |q: f32| {
        if q.is_nan() || q <= 0.0 {
            return 0u8;
        }
        let scaled = q / 257.0f32;
        if scaled >= 255.0 {
            255
        } else {
            (scaled + 0.5f32) as u8
        }
    };
    RgbaImage::from_fn(nw as u32, nh as u32, |x, y| image::Rgba(out[y as usize * nw + x as usize].map(to_char)))
}
