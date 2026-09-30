//! chromatic by crypt0lith (github.com/crypt0lith/chromatic, MIT),
//! ported from `chromatic/image/_array.py` (`img2ansi`) and
//! `chromatic/color/colorconv.py` (its 16-color lookup), with the OpenCV
//! (Apache-2.0) and Pillow (MIT-CMU) operations it runs through.
//!
//! Colored CP437: the image's gray, blurred by how far it shrinks, picks
//! one of the font's glyphs ordered by how much ink they have. The colors
//! are taken to the 16 VGA colors at full size, resized (Lanczos) and
//! taken to the VGA colors again.

use image::RgbaImage;

use super::grid::{Cell, Color, Grid};
use super::util::{drop_alpha, resample, Kernel, VGA_SGR};
use super::{Converter, ConverterInfo};

pub struct Chromatic;

static INFO: ConverterInfo = ConverterInfo {
    id: "chromatic",
    name: "chromatic",
    origin: "github.com/crypt0lith/chromatic",
    copyright: "Copyright (c) 2024 crypt0lith",
    license: "MIT",
    language: "Python",
    revision: "be4e6a5cd6df4219c1adf7f99f12b2592d21fa17",
    settings: "img2ansi(<image>, factor=80, ansi_type=\"4b\", char_set=<its CP437 set without the control-code glyphs>), default font (IBM VGA 8x16), glyphs sorted",
    adaptations: "Its default CP437 set includes the glyphs CP437 draws for control codes (☺, ♥, ↑ and so on), which an .ANS can't hold, so they're left out of the character set. For blur kernels of 50 taps or more (images over about 1,000 pixels), OpenCV convolves through a DFT, which can differ from the direct sum used here in the last bit.",
    license_text: include_str!("licenses/chromatic.txt"),
};

/// The character set (CP437 0x21–0x7E and 0x80–0xFE), sorted light to
/// dark by `sort_glyphs` against the default font, VileR's IBM VGA 8x16:
/// computed once with chromatic itself, since the sort renders the font.
const GLYPHS: [u8; 221] = [
    0xfa, 0x2e, 0xf9, 0x60, 0x3a, 0x2d, 0x27, 0x2c, 0x5f, 0xc4, 0x3b, 0x5e, 0x2f, 0x7e, 0x3d, 0xf6, 0x3e, 0x3c, 0xf8, 0x2b, 0xa9, 0xcd, 0xaa, 0x25, 0xa7, 0x7c, 0xfd, 0xc0, 0xd9, 0xf3, 0x28, 0x29,
    0xae, 0xaf, 0xf2, 0xf7, 0x22, 0xbf, 0xda, 0xbe, 0xd4, 0x69, 0x8b, 0xa6, 0xe7, 0xf0, 0xee, 0x73, 0x3f, 0x72, 0xa8, 0x5c, 0xf1, 0x7b, 0x8d, 0xfc, 0x7d, 0xa1, 0x6c, 0x63, 0x5b, 0x5d, 0x49, 0x78,
    0x7a, 0x76, 0x37, 0x8c, 0x2a, 0x43, 0x64, 0x6e, 0xad, 0x21, 0xb8, 0xd5, 0x9b, 0x74, 0x75, 0xc1, 0x32, 0x6f, 0x87, 0x31, 0x66, 0x77, 0xe2, 0xe4, 0x4a, 0xe5, 0x6a, 0x61, 0xcf, 0xf5, 0xab, 0xeb,
    0xc8, 0xec, 0x53, 0x65, 0xb3, 0x80, 0x91, 0x4c, 0x59, 0x81, 0xc2, 0xe0, 0xe6, 0x54, 0x33, 0xd3, 0xf4, 0x56, 0x97, 0xa3, 0xbc, 0xbd, 0xe3, 0x84, 0x39, 0x94, 0x5a, 0x85, 0xa0, 0x26, 0xfb, 0x50,
    0x9c, 0x86, 0xb4, 0xc3, 0x35, 0x6b, 0x89, 0x9d, 0x30, 0x47, 0x8e, 0x62, 0x34, 0xef, 0x46, 0x83, 0x36, 0x6d, 0x70, 0xa4, 0x95, 0xa2, 0xe8, 0x93, 0x41, 0x82, 0x8a, 0xe9, 0xb5, 0xc6, 0x68, 0x96,
    0xe1, 0x88, 0x67, 0x99, 0xd6, 0x71, 0xca, 0xb7, 0xed, 0x9f, 0x79, 0x90, 0x58, 0x9a, 0xd1, 0x45, 0x24, 0xac, 0x98, 0x38, 0xea, 0x55, 0x4b, 0x4f, 0xd0, 0x44, 0xb0, 0x8f, 0x52, 0xc9, 0x40, 0x48,
    0x23, 0x42, 0xbb, 0x92, 0x57, 0xd2, 0x9e, 0x51, 0xc5, 0xcb, 0x4e, 0x4d, 0xfe, 0xa5, 0xd8, 0xcc, 0xba, 0xc7, 0xce, 0xb6, 0xb9, 0xb1, 0xd7, 0xdd, 0xde, 0xb2, 0xdf, 0xdc, 0xdb,
];

/// OpenCV's `exp` for its bit-exact Gaussian kernels (`f64_exp` in
/// `softfloat.cpp`): a 64-entry table of 2^(i/64) and a polynomial.
fn cv_exp(x: f64) -> f64 {
    const TAB: [u64; 64] = [
        0x3ff0000000000000, 0x3ff02c9a3e778061, 0x3ff059b0d3158574, 0x3ff0874518759bc8,
        0x3ff0b5586cf9890f, 0x3ff0e3ec32d3d1a2, 0x3ff11301d0125b51, 0x3ff1429aaea92de0,
        0x3ff172b83c7d517b, 0x3ff1a35beb6fcb75, 0x3ff1d4873168b9aa, 0x3ff2063b88628cd6,
        0x3ff2387a6e756238, 0x3ff26b4565e27cdd, 0x3ff29e9df51fdee1, 0x3ff2d285a6e4030b,
        0x3ff306fe0a31b715, 0x3ff33c08b26416ff, 0x3ff371a7373aa9cb, 0x3ff3a7db34e59ff7,
        0x3ff3dea64c123422, 0x3ff4160a21f72e2a, 0x3ff44e086061892d, 0x3ff486a2b5c13cd0,
        0x3ff4bfdad5362a27, 0x3ff4f9b2769d2ca7, 0x3ff5342b569d4f82, 0x3ff56f4736b527da,
        0x3ff5ab07dd485429, 0x3ff5e76f15ad2148, 0x3ff6247eb03a5585, 0x3ff6623882552225,
        0x3ff6a09e667f3bcd, 0x3ff6dfb23c651a2f, 0x3ff71f75e8ec5f74, 0x3ff75feb564267c9,
        0x3ff7a11473eb0187, 0x3ff7e2f336cf4e62, 0x3ff82589994cce13, 0x3ff868d99b4492ed,
        0x3ff8ace5422aa0db, 0x3ff8f1ae99157736, 0x3ff93737b0cdc5e5, 0x3ff97d829fde4e50,
        0x3ff9c49182a3f090, 0x3ffa0c667b5de565, 0x3ffa5503b23e255d, 0x3ffa9e6b5579fdbf,
        0x3ffae89f995ad3ad, 0x3ffb33a2b84f15fb, 0x3ffb7f76f2fb5e47, 0x3ffbcc1e904bc1d2,
        0x3ffc199bdd85529c, 0x3ffc67f12e57d14b, 0x3ffcb720dcef9069, 0x3ffd072d4a07897c,
        0x3ffd5818dcfba487, 0x3ffda9e603db3285, 0x3ffdfc97337b9b5f, 0x3ffe502ee78b3ff6,
        0x3ffea4afa2a490da, 0x3ffefa1bee615a27, 0x3fff50765b6e4540, 0x3fffa7c1819e90d8,
    ];
    let a0c = f64::from_bits(0x3f83ce0f3e46f431);
    let a5 = 1.0 / a0c;
    let a4 = f64::from_bits(0x3fe62e42fefa39f1) / a0c;
    let a3 = f64::from_bits(0x3fcebfbdff82a45a) / a0c;
    let a2 = f64::from_bits(0x3fac6b08d81fec75) / a0c;
    let a1 = f64::from_bits(0x3f83b2a72b4f3cd3) / a0c;
    let a0 = f64::from_bits(0x3f55e7aa1566c2a4) / a0c;
    let prescale = f64::from_bits(0x3ff71547652b82fe) * 64.0;
    let x0 = x * prescale;
    // cvRound and f64_roundToInt, both to nearest even.
    let r = x0.round_ties_even();
    let v = r as i32;
    let t = ((v >> 6) + 1023).clamp(0, 2047) as u64;
    let buf = f64::from_bits(t << 52);
    let x0 = (x0 - r) * (1.0 / 64.0);
    buf * a0c * f64::from_bits(TAB[(v & 63) as usize]) * (((((a0 * x0 + a1) * x0 + a2) * x0 + a3) * x0 + a4) * x0 + a5)
}

/// `cv.getGaussianKernel(n, sigma)` (OpenCV's `getGaussianKernelBitExact`).
fn gaussian_kernel(n: usize, sigma: f64) -> Vec<f64> {
    let scale = -0.125 / (sigma * sigma);
    let n2 = (n - 1) / 2;
    let mut values = Vec::with_capacity(n2);
    let mut sum = 0.0;
    let mut x = 1 - n as i64;
    for _ in 0..n2 {
        let t = cv_exp((x * x) as f64 * scale);
        values.push(t);
        sum += t;
        x += 2;
    }
    sum *= 2.0;
    sum += 1.0;
    if n % 2 == 0 {
        sum += 1.0;
    }
    let mul = 1.0 / sum;
    let mut k = vec![mul; n];
    for (i, v) in values.iter().enumerate() {
        k[i] = v * mul;
        k[n - 1 - i] = v * mul;
    }
    k
}

/// OpenCV's `borderInterpolate` for BORDER_REFLECT_101.
fn reflect_101(mut p: i64, len: i64) -> usize {
    if len == 1 {
        return 0;
    }
    while p < 0 || p >= len {
        p = if p < 0 { -p } else { 2 * len - 2 - p };
    }
    p as usize
}

/// `cv.filter2D` with a 1-D kernel along one axis, BORDER_REFLECT_101: a
/// fused multiply-add per tap, in kernel order, for each pixel.
fn filter_axis(src: &[f64], w: usize, h: usize, k: &[f64], along_x: bool) -> Vec<f64> {
    let half = (k.len() / 2) as i64;
    let mut out = vec![0f64; w * h];
    if along_x {
        let idx: Vec<Vec<usize>> = (0..w as i64).map(|x| (0..k.len() as i64).map(|i| reflect_101(x + i - half, w as i64)).collect()).collect();
        for y in 0..h {
            let row = &src[y * w..(y + 1) * w];
            for x in 0..w {
                out[y * w + x] = k.iter().zip(&idx[x]).fold(0.0f64, |s, (&kv, &i)| kv.mul_add(row[i], s));
            }
        }
    } else {
        for y in 0..h {
            let acc = &mut out[y * w..(y + 1) * w];
            for (i, &kv) in k.iter().enumerate() {
                let r = reflect_101(y as i64 + i as i64 - half, h as i64);
                for (a, &v) in acc.iter_mut().zip(&src[r * w..(r + 1) * w]) {
                    *a = kv.mul_add(v, *a);
                }
            }
        }
    }
    out
}

/// `cv.resize(.., INTER_LINEAR)` on a float64 image: f32 weights, each
/// pass `fma(v0, w0, v1 * w1)`.
fn cv_resize_linear(src: &[f64], w: usize, h: usize, nw: usize, nh: usize) -> Vec<f64> {
    let table = |len: usize, n: usize| {
        let scale = 1.0 / (n as f64 / len as f64);
        (0..n)
            .map(|d| {
                let f = ((d as f64 + 0.5) * scale - 0.5) as f32;
                let s = f.floor() as i64;
                let (s, f) = if s < 0 {
                    (0, 0.0)
                } else if s >= len as i64 - 1 {
                    (len - 1, 0.0)
                } else {
                    (s as usize, f - s as f32)
                };
                (s, (1.0 - f) as f64, f as f64)
            })
            .collect::<Vec<_>>()
    };
    let (xt, yt) = (table(w, nw), table(h, nh));
    let mut tmp = vec![0f64; nw * h];
    for y in 0..h {
        for (d, &(s, a0, a1)) in xt.iter().enumerate() {
            let row = &src[y * w..(y + 1) * w];
            tmp[y * nw + d] = row[s].mul_add(a0, row[(s + 1).min(w - 1)] * a1);
        }
    }
    let mut out = vec![0f64; nw * nh];
    for (d, &(s, b0, b1)) in yt.iter().enumerate() {
        let s1 = (s + 1).min(h - 1);
        for x in 0..nw {
            out[d * nw + x] = tmp[s * nw + x].mul_add(b0, tmp[s1 * nw + x] * b1);
        }
    }
    out
}

/// `nearest_ansi_4bit_rgb`: `ANSI_4BIT_RGB_LUT` at the color's top 5 bits
/// per channel, the VGA color nearest that corner by chromatic's
/// "redmean" distance (the first on a tie).
fn nearest_vga(rgb: [u8; 3]) -> usize {
    let q = rgb.map(|v| (v >> 3) as f64 * 8.0);
    let mut best = 0;
    let mut best_d = f64::INFINITY;
    for (i, p) in VGA_SGR.iter().enumerate() {
        let p = p.map(|v| v as f64);
        let r_mean = (q[0] + p[0]) / 2.0;
        let r = (q[0] - p[0]) * (2.0 + r_mean / 256.0);
        let g = (q[1] - p[1]) * 4.0;
        let b = (q[2] - p[2]) * (2.0 + (255.0 - r_mean) / 256.0);
        let d = r * r + g * g + b * b;
        if d < best_d {
            best = i;
            best_d = d;
        }
    }
    best
}

impl Converter for Chromatic {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let rgb = drop_alpha(image);
        let (w, h) = (rgb.width() as usize, rgb.height() as usize);
        // `out_shape`: the font's space is 8×16, so it halves the rows
        // (`ceil(16 / 8)`).
        let cols = columns as usize;
        let rows = ((columns as f64 / (w as f64 / h as f64) / 2.0) as usize).max(1);
        // `to_ascii`: OpenCV's RGB2GRAY (15-bit weights), then a Gaussian
        // blur along each axis that shrinks, then a linear resize.
        let mut blur: Vec<f64> = rgb.pixels().map(|p| ((p[0] as u32 * 9798 + p[1] as u32 * 19235 + p[2] as u32 * 3735 + 16384) >> 15) as f64 / 255.0).collect();
        for (along_x, len, n) in [(false, h, rows), (true, w, cols)] {
            let s = (len as f64 / n as f64 - 1.0) / 2.0;
            if s <= 0.0 {
                continue;
            }
            let k = gaussian_kernel(2 * (4.0 * s + 0.5) as usize + 1, s);
            blur = filter_axis(&blur, w, h, &k, along_x);
        }
        let grey = cv_resize_linear(&blur, w, h, cols, rows);
        // `to_ansi`: the colors to 16 at full size, Pillow's Lanczos, and
        // to 16 again when the escape codes are written.
        let quantized = RgbaImage::from_fn(w as u32, h as u32, |x, y| {
            let p = rgb.get_pixel(x, y);
            let c = VGA_SGR[nearest_vga([p[0], p[1], p[2]])];
            image::Rgba([c[0], c[1], c[2], 255])
        });
        let colors = resample(&quantized, cols as u32, rows as u32, Kernel::Lanczos3);
        let mut grid = Grid::new(cols, rows);
        for y in 0..rows {
            for x in 0..cols {
                let g = (grey[y * cols + x] * 255.0) as u8;
                let i = (g as f64 / 255.0 * (GLYPHS.len() - 1) as f64).round_ties_even() as usize;
                let p = colors.get_pixel(x as u32, y as u32);
                grid.set(x, y, Cell::new(GLYPHS[i], Color::Ansi(nearest_vga([p[0], p[1], p[2]]) as u8), Color::Default));
            }
        }
        grid
    }
}
