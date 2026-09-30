//! img2ansi by Wes Brown (github.com/wbrown/img2ansi, BSD-3-Clause),
//! ported from `img2ansi.go` (the Brown dithering algorithm),
//! `approximatecache.go`, `bbs.go`, `rgb.go` and `imageutil/` (resize,
//! Canny edges, sharpening), run as `ansify -bbs -ice`.
//!
//! The image is resized to 2×2 pixels per cell (via a 4× intermediate that
//! Canny edge detection runs on) and sharpened. Each 2×2 block gets the
//! CP437 block glyph and foreground/background pair with the least RGB
//! error, found by brute force over the 16-color palette, and the block's
//! error is diffused Floyd–Steinberg style to its neighbors, halved at
//! edges. Blocks whose pixels map to the same palette colors reuse an
//! earlier match when it's close enough (its approximate cache).

use std::collections::HashMap;

use image::RgbaImage;

use super::grid::{cp437, Cell, Color, Grid};
use super::{Converter, ConverterInfo};

pub struct Img2AnsiWbrown;

static INFO: ConverterInfo = ConverterInfo {
    id: "img2ansi-wbrown",
    name: "img2ansi (Wes Brown)",
    origin: "github.com/wbrown/img2ansi",
    copyright: "Copyright (c) 2024, Wes Brown",
    license: "BSD-3-Clause",
    language: "Go",
    revision: "6a6c2df96d38e007c3dcb6ef5cbf8c95cc2f9929",
    settings: "ansify -bbs -ice: the 6 CP437 blocks, ansi16 palette with iCE colors, RGB color distance, cache threshold 200, scale 2",
    adaptations: "None beyond the shared width.",
    license_text: include_str!("licenses/img2ansi-wbrown.txt"),
};

/// ansi16.json, sorted by SGR code as `ReadAnsiDataFromJSON` does: the
/// foreground list is 30–37 then 90–97, the background list 40–47 then
/// 100–107, the same 16 colors in SGR order.
const PALETTE: [[u8; 3]; 16] = [
    [0x00, 0x00, 0x00],
    [0xAA, 0x00, 0x00],
    [0x00, 0xAA, 0x00],
    [0xAA, 0x55, 0x00],
    [0x00, 0x00, 0xAA],
    [0xAA, 0x00, 0xAA],
    [0x00, 0xAA, 0xAA],
    [0xAA, 0xAA, 0xAA],
    [0x55, 0x55, 0x55],
    [0xFF, 0x55, 0x55],
    [0x55, 0xFF, 0x55],
    [0xFF, 0xFF, 0x55],
    [0x55, 0x55, 0xFF],
    [0xFF, 0x55, 0xFF],
    [0x55, 0xFF, 0xFF],
    [0xFF, 0xFF, 0xFF],
];

/// BBSBlocks: CP437 byte and quadrants (TL, TR, BL, BR) in foreground.
const BLOCKS: [(u8, [bool; 4]); 6] = [
    (b' ', [false, false, false, false]),
    (cp437::UPPER_HALF, [true, true, false, false]),
    (cp437::LOWER_HALF, [false, false, true, true]),
    (cp437::LEFT_HALF, [true, false, true, false]),
    (cp437::RIGHT_HALF, [false, true, false, true]),
    (cp437::FULL_BLOCK, [true, true, true, true]),
];

const SCALE_FACTOR: f64 = 2.0;
const CACHE_THRESHOLD: f64 = 200.0;

type Rgb = [u8; 3];

/// RGBMethod.Distance.
fn distance(a: Rgb, b: Rgb) -> f64 {
    let d = |i: usize| (a[i] as i32 - b[i] as i32).pow(2);
    ((d(0) + d(1) + d(2)) as f64).sqrt()
}

fn block_error(block: &[Rgb; 4], quad: [bool; 4], fg: Rgb, bg: Rgb, edge: bool) -> f64 {
    let mut total = 0.0;
    for i in 0..4 {
        total += distance(block[i], if quad[i] { fg } else { bg });
    }
    if edge {
        total *= 0.5;
    }
    total
}

/// The closest palette color (the precomputed table's linear scan).
fn closest(c: Rgb) -> usize {
    let mut best = 0;
    let mut best_d = distance(c, PALETTE[0]);
    for (i, &p) in PALETTE.iter().enumerate().skip(1) {
        let d = distance(c, p);
        if d < best_d {
            best = i;
            best_d = d;
        }
    }
    best
}

struct Match {
    block: usize,
    fg: usize,
    bg: usize,
    error: f64,
}

#[derive(Default)]
struct Search {
    cache: HashMap<[u8; 4], Vec<Match>>,
}

impl Search {
    /// FindBestBlockRepresentation: (block index, fg, bg) palette indexes.
    fn best(&mut self, block: &[Rgb; 4], edge: bool) -> (usize, usize, usize) {
        let key = [closest(block[0]) as u8, closest(block[1]) as u8, closest(block[2]) as u8, closest(block[3]) as u8];
        // getCacheEntry
        let threshold = if edge { CACHE_THRESHOLD * 0.7 } else { CACHE_THRESHOLD };
        if let Some(matches) = self.cache.get(&key) {
            let mut lowest = f64::MAX;
            let mut found = None;
            for m in matches {
                let error = block_error(block, BLOCKS[m.block].1, PALETTE[m.fg], PALETTE[m.bg], edge);
                let exact = (error - m.error).abs() < 0.001;
                if error < lowest && (error < threshold || exact) {
                    lowest = error;
                    found = Some((m.block, m.fg, m.bg));
                }
            }
            if let Some(f) = found {
                return f;
            }
        }
        // Brute force (16 colors ≤ 32).
        let mut best = (0, 0, 0);
        let mut min = f64::MAX;
        for (b, &(_, quad)) in BLOCKS.iter().enumerate() {
            for fg in 0..16 {
                for bg in 0..16 {
                    if PALETTE[fg] != PALETTE[bg] {
                        let e = block_error(block, quad, PALETTE[fg], PALETTE[bg], edge);
                        if e < min {
                            min = e;
                            best = (b, fg, bg);
                        }
                    }
                }
            }
        }
        // addCacheEntry
        let error = block_error(block, BLOCKS[best.0].1, PALETTE[best.1], PALETTE[best.2], edge);
        self.cache.entry(key).or_default().push(Match {
            block: best.0,
            fg: best.1,
            bg: best.2,
            error,
        });
        best
    }
}

/// A premultiplied 8-bit RGBA image, as Go's `*image.RGBA`.
struct Premul {
    w: usize,
    h: usize,
    px: Vec<[u8; 4]>,
}

impl Premul {
    /// `RGBAImageFromImage`: each pixel through `color.RGBAModel`.
    fn from_straight(img: &RgbaImage) -> Premul {
        let px = img
            .pixels()
            .map(|p| {
                let a = p[3] as u32;
                let m = |c: u8| ((((c as u32) * 0x101 * a) / 0xff) >> 8) as u8;
                [m(p[0]), m(p[1]), m(p[2]), p[3]]
            })
            .collect();
        Premul {
            w: img.width() as usize,
            h: img.height() as usize,
            px,
        }
    }

    fn rgb(&self, x: usize, y: usize) -> Rgb {
        let p = self.px[y * self.w + x];
        [p[0], p[1], p[2]]
    }
}

/// golang.org/x/image/draw's kernels, exactly as it evaluates them.
#[derive(Clone, Copy)]
enum GoKernel {
    BiLinear,
    CatmullRom,
}

impl GoKernel {
    fn support(self) -> f64 {
        match self {
            GoKernel::BiLinear => 1.0,
            GoKernel::CatmullRom => 2.0,
        }
    }

    fn at(self, t: f64) -> f64 {
        match self {
            GoKernel::BiLinear => 1.0 - t,
            GoKernel::CatmullRom => {
                if t < 1.0 {
                    (1.5 * t - 2.5) * t * t + 1.0
                } else {
                    ((-0.5 * t + 2.5) * t - 4.0) * t + 2.0
                }
            }
        }
    }
}

struct Source {
    contribs: Vec<(usize, f64)>,
    inv_total: f64,
    inv_total_ffff: f64,
}

/// x/image/draw's `newDistrib`.
fn distrib(k: GoKernel, dw: usize, sw: usize) -> Vec<Source> {
    let scale = sw as f64 / dw as f64;
    let (mut half, mut arg_scale) = (k.support(), 1.0);
    if scale > 1.0 {
        half *= scale;
        arg_scale = 1.0 / scale;
    }
    (0..dw)
        .map(|x| {
            let center = (x as f64 + 0.5) * scale - 0.5;
            let i = ((center - half).floor() as i64).max(0);
            let mut j = (center + half).ceil() as i64;
            if j > sw as i64 {
                j = (sw as i64).max(i);
            }
            let mut contribs = Vec::new();
            let mut total = 0.0;
            for coord in i..j {
                let t = ((center - coord as f64) * arg_scale).abs();
                if t >= k.support() {
                    continue;
                }
                let w = k.at(t);
                if w == 0.0 {
                    continue;
                }
                total += w;
                contribs.push((coord as usize, w));
            }
            let inv_total = 1.0 / total;
            Source {
                contribs,
                inv_total,
                inv_total_ffff: inv_total / 65535.0,
            }
        })
        .collect()
}

/// x/image/draw's `ftou`.
fn ftou(f: f64) -> u16 {
    let i = (65535.0 * f + 0.5) as i32;
    i.clamp(0, 0xffff) as u16
}

/// `Kernel.Scale` of an `*image.RGBA` into a new one (op Over on a
/// transparent destination, which equals Src): a horizontal pass into a
/// float buffer, then a vertical pass.
fn scale_rgba(src: &Premul, dw: usize, dh: usize, k: GoKernel) -> Premul {
    let hs = distrib(k, dw, src.w);
    let vs = distrib(k, dh, src.h);
    let mut tmp = vec![[0f64; 4]; dw * src.h];
    for y in 0..src.h {
        for (x, s) in hs.iter().enumerate() {
            let mut p = [0f64; 4];
            for &(coord, w) in &s.contribs {
                let v = src.px[y * src.w + coord];
                for c in 0..4 {
                    p[c] += (v[c] as u32 * 0x101) as f64 * w;
                }
            }
            tmp[y * dw + x] = p.map(|v| v * s.inv_total_ffff);
        }
    }
    let mut px = vec![[0u8; 4]; dw * dh];
    for x in 0..dw {
        for (y, s) in vs.iter().enumerate() {
            let mut p = [0f64; 4];
            for &(coord, w) in &s.contribs {
                let t = tmp[coord * dw + x];
                for c in 0..4 {
                    p[c] += t[c] * w;
                }
            }
            for c in 0..3 {
                if p[c] > p[3] {
                    p[c] = p[3];
                }
            }
            px[y * dw + x] = p.map(|v| (ftou(v * s.inv_total) >> 8) as u8);
        }
    }
    Premul { w: dw, h: dh, px }
}

/// `Kernel.Scale` of an `*image.Gray` into a new one.
fn scale_gray(src: &[u8], sw: usize, sh: usize, dw: usize, dh: usize, k: GoKernel) -> Vec<u8> {
    let hs = distrib(k, dw, sw);
    let vs = distrib(k, dh, sh);
    let mut tmp = vec![0f64; dw * sh];
    for y in 0..sh {
        for (x, s) in hs.iter().enumerate() {
            let mut p = 0.0;
            for &(coord, w) in &s.contribs {
                p += (src[y * sw + coord] as u32 * 0x101) as f64 * w;
            }
            tmp[y * dw + x] = p * s.inv_total_ffff;
        }
    }
    let mut out = vec![0u8; dw * dh];
    for x in 0..dw {
        for (y, s) in vs.iter().enumerate() {
            let (mut p, mut a) = (0.0, 0.0);
            for &(coord, w) in &s.contribs {
                p += tmp[coord * dw + x] * w;
                a += w; // the temporary alpha is 1
            }
            if p > a {
                p = a;
            }
            out[y * dw + x] = (ftou(p * s.inv_total) >> 8) as u8;
        }
    }
    out
}

/// `clampUint8`: rounds half away from zero.
fn clamp_u8(v: f64) -> u8 {
    if v < 0.0 {
        0
    } else if v > 255.0 {
        255
    } else {
        v.round() as u8
    }
}

/// `ConvolveGrayFloat` with border replication.
fn convolve_f(img: &[f64], w: usize, h: usize, k: &[&[f64]]) -> Vec<f64> {
    let (kh, kw) = (k.len(), k[0].len());
    let mut out = vec![0.0; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0;
            for (ky, row) in k.iter().enumerate() {
                for (kx, &kv) in row.iter().enumerate() {
                    let sx = (x as i64 + kx as i64 - (kw / 2) as i64).clamp(0, w as i64 - 1) as usize;
                    let sy = (y as i64 + ky as i64 - (kh / 2) as i64).clamp(0, h as i64 - 1) as usize;
                    sum += img[sy * w + sx] * kv;
                }
            }
            out[y * w + x] = sum;
        }
    }
    out
}

/// `CannyDefault`: 5×5 Gaussian, Sobel, non-maximum suppression, double
/// threshold (50, 150) and hysteresis.
fn canny(gray: &[u8], w: usize, h: usize) -> Vec<u8> {
    const G5: [[f64; 5]; 5] = [
        [2.0, 4.0, 5.0, 4.0, 2.0],
        [4.0, 9.0, 12.0, 9.0, 4.0],
        [5.0, 12.0, 15.0, 12.0, 5.0],
        [4.0, 9.0, 12.0, 9.0, 4.0],
        [2.0, 4.0, 5.0, 4.0, 2.0],
    ];
    let g5: Vec<Vec<f64>> = G5.iter().map(|r| r.iter().map(|v| v / 159.0).collect()).collect();
    let g5r: Vec<&[f64]> = g5.iter().map(|r| r.as_slice()).collect();
    let src: Vec<f64> = gray.iter().map(|&v| v as f64).collect();
    // GaussianBlurGray rounds back to 8 bits.
    let blurred: Vec<f64> = convolve_f(&src, w, h, &g5r).into_iter().map(|v| clamp_u8(v) as f64).collect();
    let gx = convolve_f(&blurred, w, h, &[&[-1.0, 0.0, 1.0], &[-2.0, 0.0, 2.0], &[-1.0, 0.0, 1.0]]);
    let gy = convolve_f(&blurred, w, h, &[&[-1.0, -2.0, -1.0], &[0.0, 0.0, 0.0], &[1.0, 2.0, 1.0]]);
    let mag: Vec<f64> = (0..w * h).map(|i| (gx[i] * gx[i] + gy[i] * gy[i]).sqrt()).collect();
    let dir: Vec<f64> = (0..w * h).map(|i| gy[i].atan2(gx[i])).collect();
    let mut sup = vec![0.0; w * h];
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            let mut angle = dir[y * w + x] * 180.0 / std::f64::consts::PI;
            if angle < 0.0 {
                angle += 180.0;
            }
            let m = |x: usize, y: usize| mag[y * w + x];
            let (q, r) = if (0.0..22.5).contains(&angle) || (157.5..=180.0).contains(&angle) {
                (m(x + 1, y), m(x - 1, y))
            } else if (22.5..67.5).contains(&angle) {
                (m(x + 1, y + 1), m(x - 1, y - 1))
            } else if (67.5..112.5).contains(&angle) {
                (m(x, y + 1), m(x, y - 1))
            } else {
                (m(x - 1, y + 1), m(x + 1, y - 1))
            };
            let v = mag[y * w + x];
            if v >= q && v >= r {
                sup[y * w + x] = v;
            }
        }
    }
    let strong: Vec<bool> = sup.iter().map(|&v| v >= 150.0).collect();
    let weak: Vec<bool> = sup.iter().map(|&v| (50.0..150.0).contains(&v)).collect();
    let mut edges: Vec<u8> = strong.iter().map(|&s| if s { 255 } else { 0 }).collect();
    let mut changed = true;
    while changed {
        changed = false;
        for y in 1..h.saturating_sub(1) {
            for x in 1..w.saturating_sub(1) {
                if weak[y * w + x] && edges[y * w + x] == 0 {
                    let near = (0..3).any(|dy| (0..3).any(|dx| edges[(y + dy - 1) * w + (x + dx - 1)] == 255));
                    if near {
                        edges[y * w + x] = 255;
                        changed = true;
                    }
                }
            }
        }
    }
    edges
}

/// `Sharpen`: the mild 3×3 kernel, border replicated, alpha set opaque.
fn sharpen(img: &Premul) -> Premul {
    let k: [&[f64]; 3] = [&[0.0, -0.5, 0.0], &[-0.5, 3.0, -0.5], &[0.0, -0.5, 0.0]];
    let (w, h) = (img.w, img.h);
    let mut out = vec![[0u8; 4]; w * h];
    let channels: Vec<Vec<f64>> = (0..3).map(|c| img.px.iter().map(|p| p[c] as f64).collect()).collect();
    let conv: Vec<Vec<f64>> = channels.iter().map(|ch| convolve_f(ch, w, h, &k)).collect();
    for i in 0..w * h {
        out[i] = [clamp_u8(conv[0][i]), clamp_u8(conv[1][i]), clamp_u8(conv[2][i]), 255];
    }
    Premul { w, h, px: out }
}

impl Converter for Img2AnsiWbrown {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let img = Premul::from_straight(image);
        let aspect = img.w as f64 / img.h as f64;
        let width = columns as usize;
        let height = ((width as f64 / aspect / SCALE_FACTOR) as usize).max(1);

        // PrepareForANSI
        let inter = scale_rgba(&img, width * 4, height * 4, GoKernel::CatmullRom);
        let gray: Vec<u8> = inter
            .px
            .iter()
            .map(|p| ((299 * p[0] as u32 + 587 * p[1] as u32 + 114 * p[2] as u32 + 500) / 1000).min(255) as u8)
            .collect();
        let edges_full = canny(&gray, inter.w, inter.h);
        let mut resized = scale_rgba(&inter, width * 2, height * 2, GoKernel::CatmullRom);
        let edges = scale_gray(&edges_full, inter.w, inter.h, width * 2, height * 2, GoKernel::BiLinear);
        resized = sharpen(&resized);

        // BrownDitherForBlocks
        let (iw, ih) = (resized.w, resized.h);
        let mut search = Search::default();
        let mut grid = Grid::new(width, height);
        for by in 0..height {
            for bx in 0..width {
                let block = [resized.rgb(bx * 2, by * 2), resized.rgb(bx * 2 + 1, by * 2), resized.rgb(bx * 2, by * 2 + 1), resized.rgb(bx * 2 + 1, by * 2 + 1)];
                let e = |x: usize, y: usize| edges[y * iw + x] > 128;
                let edge = e(bx * 2, by * 2) || e(bx * 2 + 1, by * 2) || e(bx * 2, by * 2 + 1) || e(bx * 2 + 1, by * 2 + 1);
                let (b, fg, bg) = search.best(&block, edge);
                grid.set(bx, by, Cell::new(BLOCKS[b].0, Color::Ansi(fg as u8), Color::Ansi(bg as u8)));

                let quad = BLOCKS[b].1;
                for (i, &c) in block.iter().enumerate() {
                    let (y, x) = (by * 2 + i / 2, bx * 2 + i % 2);
                    let target = if quad[i] { PALETTE[fg] } else { PALETTE[bg] };
                    let err = [c[0] as i16 - target[0] as i16, c[1] as i16 - target[1] as i16, c[2] as i16 - target[2] as i16];
                    let scale = if edge { 0.5 } else { 1.0 };
                    let mut diffuse = |y: i64, x: i64, factor: f64| {
                        if y >= 0 && (y as usize) < ih && x >= 0 && (x as usize) < iw {
                            let p = &mut resized.px[y as usize * iw + x as usize];
                            for ch in 0..3 {
                                p[ch] = (p[ch] as f64 + err[ch] as f64 * factor * scale).clamp(0.0, 255.0) as u8;
                            }
                            p[3] = 255;
                        }
                    };
                    let (y, x) = (y as i64, x as i64);
                    diffuse(y, x + 1, 7.0 / 16.0);
                    diffuse(y + 1, x - 1, 3.0 / 16.0);
                    diffuse(y + 1, x, 5.0 / 16.0);
                    diffuse(y + 1, x + 1, 1.0 / 16.0);
                }
            }
        }
        grid
    }
}
