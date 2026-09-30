//! image-to-ascii by Ionică Bizău (github.com/IonicaBizau/image-to-ascii,
//! MIT), ported from `lib/index.js` and the packages it's built from, all
//! his and MIT: compute-size, asciify-pixel, pixel-class, pixel-bg,
//! pixel-white-bg and couleurs. It resizes with lwip (MIT, Eyal Arubas),
//! whose `resize` is CImg 1.6.6's; the parts of that ported here are only
//! its arithmetic (CImg: CeCILL-C).
//!
//! Colored ASCII: one character per pixel from ` .,:;i1tfLCG08@` by the
//! sum of its channels, in its 24-bit color, transparency blended over
//! white.

use image::RgbaImage;

use super::grid::{Cell, Color, Grid};
use super::{Converter, ConverterInfo};

pub struct ImageToAscii;

static INFO: ConverterInfo = ConverterInfo {
    id: "image-to-ascii",
    name: "image-to-ascii",
    origin: "github.com/IonicaBizau/image-to-ascii",
    copyright: "Copyright (c) 2014-25 Ionică Bizău",
    license: "MIT",
    language: "JavaScript",
    revision: "e74fe929b73e8a714749a168b4f4ba20d7bb2037",
    settings: "imageToAscii(<image>, { size: { width: 40 } }) on a 24-bit-color terminal tall enough for the image (it doubles widths for 1:2 cells, so 40 makes 80 columns)",
    adaptations: "None.",
    license_text: include_str!("licenses/image-to-ascii.txt"),
};

/// asciify-pixel's default `pixels`.
const PIXELS: &[u8] = b" .,:;i1tfLCG08@";

/// JavaScript's `Math.round`: halves go up.
fn js_round(x: f64) -> f64 {
    let f = x.floor();
    if x - f >= 0.5 {
        f + 1.0
    } else {
        f
    }
}

/// CImg 1.6.6's `_cimg_lanczos` (a = 2), in f32.
fn lanczos(x: f32) -> f32 {
    if x <= -2.0 || x >= 2.0 {
        return 0.0;
    }
    let a = std::f32::consts::PI * x;
    let b = 0.5 * a;
    if x != 0.0 {
        a.sin() * b.sin() / (a * b)
    } else {
        1.0
    }
}

/// One axis of CImg 1.6.6's `resize(.., 6)` (Lanczos), on `lines` lines
/// of `len` values, `n` out: nearest from a single value, a moving
/// average when shrinking, Lanczos when growing; truncated to bytes.
fn resize_axis(line: &[u8], n: usize) -> Vec<u8> {
    let len = line.len();
    if len == 1 {
        return vec![line[0]; n];
    }
    if len > n {
        // Type 2, moving average.
        let mut tmp = vec![0f32; n];
        let (mut a, mut b, mut c, mut s, mut t) = (len * n, len, n, 0, 0);
        while a > 0 {
            let d = b.min(c);
            a -= d;
            b -= d;
            c -= d;
            tmp[t] += line[s] as f32 * d as f32;
            if b == 0 {
                tmp[t] /= len as f32;
                t += 1;
                b = len;
            }
            if c == 0 {
                s += 1;
                c = n;
            }
        }
        return tmp.iter().map(|&v| v as u8).collect();
    }
    // Type 6, growing, boundary conditions 0.
    let f = if n > 1 { (len as f32 - 1.0) / (n as f32 - 1.0) } else { 0.0 };
    let mut curr = 0f32;
    let mut p = 0usize;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let t = curr - (curr as u32) as f32;
        let old = curr;
        curr += f;
        let step = curr as u32 as usize - old as u32 as usize;
        let (w0, w1, w2, w3, w4) = (lanczos(t + 2.0), lanczos(t + 1.0), lanczos(t), lanczos(t - 1.0), lanczos(t - 2.0));
        let v = |i: usize| line[i] as f32;
        let val2 = v(p);
        let val1 = if p >= 1 { v(p - 1) } else { val2 };
        let val0 = if p > 1 { v(p - 2) } else { val1 };
        let val3 = if p + 2 <= len { v(p + 1) } else { val2 };
        let val4 = if p + 2 < len { v(p + 2) } else { val3 };
        let val = (val0 * w0 + val1 * w1 + val2 * w2 + val3 * w3 + val4 * w4) / (w1 + w2 + w3 + w4);
        out.push(val.clamp(0.0, 255.0) as u8);
        p += step;
    }
    out
}

/// CImg's `resize(nw, nh, -100, -100, 6)` on 4-channel pixels: along x,
/// then along y, each channel alone.
fn cimg_resize(px: &[[u8; 4]], w: usize, h: usize, nw: usize, nh: usize) -> Vec<[u8; 4]> {
    let mut cur = px.to_vec();
    if nw != w {
        let mut next = vec![[0u8; 4]; nw * h];
        for y in 0..h {
            for c in 0..4 {
                let line: Vec<u8> = (0..w).map(|x| cur[y * w + x][c]).collect();
                for (x, v) in resize_axis(&line, nw).into_iter().enumerate() {
                    next[y * nw + x][c] = v;
                }
            }
        }
        cur = next;
    }
    if nh != h {
        let mut next = vec![[0u8; 4]; nw * nh];
        for x in 0..nw {
            for c in 0..4 {
                let line: Vec<u8> = (0..h).map(|y| cur[y * nw + x][c]).collect();
                for (y, v) in resize_axis(&line, nh).into_iter().enumerate() {
                    next[y * nw + x][c] = v;
                }
            }
        }
        cur = next;
    }
    cur
}

impl Converter for ImageToAscii {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let (w, h) = (image.width() as usize, image.height() as usize);
        // compute-size: `size.width` in pixels, the height by the aspect
        // ratio, then both times `px_size` (width 1 / 0.5); lwip truncates.
        let half = (columns / 2) as f64;
        let nw = columns as usize;
        let nh = ((h as f64 * half / w as f64) as usize).max(1);
        // lwip's `toRGBA`: alpha to 0–100.
        let px: Vec<[u8; 4]> = image.pixels().map(|p| [p[0], p[1], p[2], ((p[3] as f64 / 255.0) * 100.0) as u8]).collect();
        let img = cimg_resize(&px, w, h, nw, nh);
        let precision = 255.0 * 3.0 / (PIXELS.len() - 1) as f64;
        let mut grid = Grid::new(nw, nh);
        for (i, p) in img.iter().enumerate() {
            // pixel-class: alpha over 1 is a percentage (so 1% counts as 1).
            let a = if p[3] > 1 { p[3] as f64 / 100.0 } else { p[3] as f64 };
            let mut rgb = [p[0], p[1], p[2]].map(|v| v as f64);
            // pixel-white-bg.
            if a != 1.0 {
                rgb = rgb.map(|v| (1.0 - a) * 255.0 + a * v);
            }
            let value = rgb[0] + rgb[1] + rgb[2];
            let ch = PIXELS[js_round(value / precision) as usize];
            // couleurs, through color-convert's `rgb.hex`.
            let [r, g, b] = rgb.map(|v| (js_round(v) as i64 & 0xff) as u8);
            grid.set(i % nw, i / nw, Cell::new(ch, Color::Rgb(r, g, b), Color::Default));
        }
        grid
    }
}
