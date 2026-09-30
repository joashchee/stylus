//! TerminalImageViewer (tiv) by Stefan Haustein and Aaron Liu
//! (github.com/stefanhaustein/TerminalImageViewer, Apache-2.0 or GPL-3.0,
//! used under Apache-2.0), ported from `src/tiv_lib.cpp` (`findCharData`,
//! `createCharData`), `src/tiv.cpp` (`printImage`, sizing, loading) and
//! the parts of CImg's `resize` it uses (CImg: CeCILL-C, not ported
//! beyond that arithmetic).
//!
//! Each cell covers 4×8 pixels. Those are split into two colors (the two
//! commonest, or by the channel with the widest range), and the glyph
//! whose 4×8 bitmap, or its inverse, differs from that split in the
//! fewest pixels wins, in 24-bit color.

use image::RgbaImage;

use super::grid::{cp437, Cell, Color, Grid};
use super::{Converter, ConverterInfo};

pub struct Tiv;

static INFO: ConverterInfo = ConverterInfo {
    id: "tiv",
    name: "TerminalImageViewer",
    origin: "github.com/stefanhaustein/TerminalImageViewer",
    copyright: "Copyright (c) 2017-2023, Stefan Haustein, Aaron Liu",
    license: "Apache-2.0",
    language: "C++",
    revision: "d93c9d1088a7022b4e49413cc752a00b6c9ced61",
    settings: "tiv -w 80 -h 10000 (24-bit color, white behind transparency)",
    adaptations: "Its glyph table is cut to the glyphs CP437 has (space, ▄, ▌, ─, │, and by inversion ▀, ▐, █); the rest (quadrants, eighths, heavy lines) can't go in an .ANS. It only shrinks, so an image narrower than 320 pixels is enlarged to 80 columns here (CImg's cubic).",
    license_text: include_str!("licenses/tiv.txt"),
};

/// BITMAPS, kept to the entries CP437 can show, in their order: the 4×8
/// bitmap (one hex digit per row) and the CP437 byte.
const BITMAPS: [(u32, u8); 7] = [
    (0x00000000, b' '),               // U+00A0
    (0x0000ffff, cp437::LOWER_HALF),  // U+2584 lower 1/2
    (0xcccccccc, cp437::LEFT_HALF),   // U+258C left 1/2
    (0x000f0000, 0xC4),               // U+2500 light horizontal
    (0x0000f000, 0xC4),               //
    (0x44444444, 0xB3),               // U+2502 light vertical
    (0x22222222, 0xB3),               //
];

struct CharData {
    fg: [i32; 3],
    bg: [i32; 3],
    ch: u8,
}

fn channel(rgb: u32, i: usize) -> i32 {
    ((rgb >> ((2 - i) * 8)) & 255) as i32
}

fn create_char_data(px: &dyn Fn(u32, u32) -> u32, x0: u32, y0: u32, ch: u8, pattern: u32) -> CharData {
    let (mut fg, mut bg) = ([0i32; 3], [0i32; 3]);
    let (mut fg_count, mut bg_count) = (0, 0);
    let mut mask = 0x8000_0000u32;
    for y in 0..8 {
        for x in 0..4 {
            let avg = if pattern & mask != 0 {
                fg_count += 1;
                &mut fg
            } else {
                bg_count += 1;
                &mut bg
            };
            let rgb = px(x0 + x, y0 + y);
            for (i, a) in avg.iter_mut().enumerate() {
                *a += channel(rgb, i);
            }
            mask >>= 1;
        }
    }
    for i in 0..3 {
        if bg_count != 0 {
            bg[i] /= bg_count;
        }
        if fg_count != 0 {
            fg[i] /= fg_count;
        }
    }
    CharData { fg, bg, ch }
}

fn find_char_data(px: &dyn Fn(u32, u32) -> u32, x0: u32, y0: u32) -> CharData {
    let (mut min, mut max) = ([255i32; 3], [0i32; 3]);
    let mut count_per_color = std::collections::BTreeMap::<u32, i32>::new();
    for y in 0..8 {
        for x in 0..4 {
            let rgb = px(x0 + x, y0 + y);
            for i in 0..3 {
                min[i] = min[i].min(channel(rgb, i));
                max[i] = max[i].max(channel(rgb, i));
            }
            *count_per_color.entry(rgb).or_default() += 1;
        }
    }
    // A multimap from count to color, read from the top: highest count
    // first, and among equal counts the last inserted (largest color).
    let mut by_count: Vec<(i32, u32)> = count_per_color.iter().map(|(&c, &n)| (n, c)).collect();
    by_count.sort_by_key(|&(n, _)| n); // stable: equal counts keep color order
    let mut it = by_count.iter().rev();
    let &(n1, mut color1) = it.next().expect("32 pixels");
    let mut count2 = n1;
    let mut color2 = color1;
    if let Some(&(n, c)) = it.next() {
        count2 += n;
        color2 = c;
    }
    let mut bits = 0u32;
    let direct = count2 > (8 * 4) / 2;
    if direct {
        for y in 0..8 {
            for x in 0..4 {
                bits <<= 1;
                let rgb = px(x0 + x, y0 + y);
                let (mut d1, mut d2) = (0, 0);
                for i in 0..3 {
                    let c = channel(rgb, i);
                    d1 += (channel(color1, i) - c).pow(2);
                    d2 += (channel(color2, i) - c).pow(2);
                }
                if d1 > d2 {
                    bits |= 1;
                }
            }
        }
    } else {
        let (mut split_index, mut best_split) = (0, 0);
        for i in 0..3 {
            if max[i] - min[i] > best_split {
                best_split = max[i] - min[i];
                split_index = i;
            }
        }
        let split_value = min[split_index] + best_split / 2;
        for y in 0..8 {
            for x in 0..4 {
                bits <<= 1;
                if channel(px(x0 + x, y0 + y), split_index) > split_value {
                    bits |= 1;
                }
            }
        }
    }
    let mut best_diff = 8;
    let mut best_pattern = 0x0000ffffu32;
    let mut ch = cp437::LOWER_HALF;
    let mut inverted = false;
    for &(bitmap, glyph) in &BITMAPS {
        let mut pattern = bitmap;
        for _ in 0..2 {
            let diff = (pattern ^ bits).count_ones();
            if diff < best_diff {
                best_pattern = bitmap;
                ch = glyph;
                best_diff = diff;
                inverted = best_pattern != pattern;
            }
            pattern = !pattern;
        }
    }
    if direct {
        if inverted {
            std::mem::swap(&mut color1, &mut color2);
        }
        return CharData {
            fg: [0, 1, 2].map(|i| channel(color2, i)),
            bg: [0, 1, 2].map(|i| channel(color1, i)),
            ch,
        };
    }
    create_char_data(px, x0, y0, ch, best_pattern)
}

/// CImg's moving-average shrink along one axis (`resize` type 2), f32
/// sums, truncated back to bytes.
fn average_axis(src: &[[u8; 3]], w: usize, h: usize, n: usize, along_x: bool) -> Vec<[u8; 3]> {
    let (len, lines) = if along_x { (w, h) } else { (h, w) };
    let (ow, oh) = if along_x { (n, h) } else { (w, n) };
    let mut out = vec![[0u8; 3]; ow * oh];
    for line in 0..lines {
        let at = |i: usize| if along_x { line * w + i } else { i * w + line };
        let put = |i: usize| if along_x { line * ow + i } else { i * ow + line };
        let mut tmp = vec![[0f32; 3]; n];
        let (mut a, mut b, mut c, mut s, mut t) = (len * n, len, n, 0, 0);
        while a > 0 {
            let d = b.min(c);
            a -= d;
            b -= d;
            c -= d;
            for ch in 0..3 {
                tmp[t][ch] += src[at(s)][ch] as f32 * d as f32;
            }
            if b == 0 {
                for ch in 0..3 {
                    tmp[t][ch] /= len as f32;
                }
                t += 1;
                b = len;
            }
            if c == 0 {
                s += 1;
                c = n;
            }
        }
        for (i, v) in tmp.iter().enumerate() {
            out[put(i)] = v.map(|x| x as u8);
        }
    }
    out
}

/// CImg's cubic enlargement along one axis (`resize` type 5, growing).
fn cubic_axis(src: &[[u8; 3]], w: usize, h: usize, n: usize, along_x: bool) -> Vec<[u8; 3]> {
    let (len, lines) = if along_x { (w, h) } else { (h, w) };
    let (ow, oh) = if along_x { (n, h) } else { (w, n) };
    let f = if n > 1 { (len as f64 - 1.0) / (n as f64 - 1.0) } else { 0.0 };
    let mut offs = Vec::with_capacity(n);
    let (mut curr, mut old) = (0f64, 0f64);
    for _ in 0..n {
        let t = curr - (curr as u32) as f64;
        old = curr;
        curr = (len as f64 - 1.0).min(curr + f);
        offs.push((t, (curr as u32 - old as u32) as usize));
    }
    let _ = old;
    let mut out = vec![[0u8; 3]; ow * oh];
    for line in 0..lines {
        let at = |i: usize| if along_x { line * w + i } else { i * w + line };
        let mut p = 0usize;
        for (i, &(t, step)) in offs.iter().enumerate() {
            let v = |k: usize, ch: usize| src[at(k)][ch] as f64;
            let mut px = [0u8; 3];
            for ch in 0..3 {
                let v1 = v(p, ch);
                let v0 = if p > 0 { v(p - 1, ch) } else { v1 };
                let v2 = if p + 2 <= len { v(p + 1, ch) } else { v1 };
                let v3 = if p + 2 < len { v(p + 2, ch) } else { v2 };
                let val = v1 + 0.5 * (t * (-v0 + v2) + t * t * (2.0 * v0 - 5.0 * v1 + 4.0 * v2 - v3) + t * t * t * (-v0 + 3.0 * v1 - 3.0 * v2 + v3));
                px[ch] = val.clamp(0.0, 255.0) as u8;
            }
            let o = if along_x { line * ow + i } else { i * ow + line };
            out[o] = px;
            p += step;
        }
    }
    out
}

/// CImg's `resize(nw, nh, -100, -100, 5)` on an RGB image.
fn cimg_resize(src: Vec<[u8; 3]>, w: usize, h: usize, nw: usize, nh: usize) -> Vec<[u8; 3]> {
    let axis = |src: &[[u8; 3]], w: usize, h: usize, n: usize, along_x: bool| {
        let len = if along_x { w } else { h };
        if len == 1 {
            // Nearest neighbor from one pixel: repeat it.
            let (ow, oh) = if along_x { (n, h) } else { (w, n) };
            (0..ow * oh).map(|i| if along_x { src[i / ow * w] } else { src[i % ow] }).collect()
        } else if len > n {
            average_axis(src, w, h, n, along_x)
        } else {
            cubic_axis(src, w, h, n, along_x)
        }
    };
    let x = if nw != w { axis(&src, w, h, nw, true) } else { src };
    if nh != h {
        axis(&x, nw, h, nh, false)
    } else {
        x
    }
}

impl Converter for Tiv {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let (w, h) = (image.width() as usize, image.height() as usize);
        // load_rgb_CImg: RGBA drawn over white with the alpha as a mask.
        let rgb: Vec<[u8; 3]> = image
            .pixels()
            .map(|p| {
                let m = p[3] as f32;
                [0, 1, 2].map(|c| ((m * p[c] as f32 + 255.0 * (255.0 - m)) / 255.0) as u8)
            })
            .collect();
        let (max_w, max_h) = (4 * columns as usize, 8 * 10000usize);
        // fitted_within, which tiv applies only to images too big.
        let scale = (max_w as f64 / w as f64).min(max_h as f64 / h as f64);
        let (nw, nh) = (((w as f64 * scale) as usize).max(1), ((h as f64 * scale) as usize).max(1));
        let img = cimg_resize(rgb, w, h, nw, nh);
        let px = |x: u32, y: u32| {
            let p = img[y as usize * nw + x as usize];
            (p[0] as u32) << 16 | (p[1] as u32) << 8 | p[2] as u32
        };
        let (cols, rows) = (nw / 4, (nh / 8).max(1));
        let mut grid = Grid::new(cols.max(1), rows);
        for row in 0..nh / 8 {
            for col in 0..cols {
                let d = find_char_data(&px, col as u32 * 4, row as u32 * 8);
                let c = |v: [i32; 3]| Color::Rgb(v[0].clamp(0, 255) as u8, v[1].clamp(0, 255) as u8, v[2].clamp(0, 255) as u8);
                grid.set(col, row, Cell::new(d.ch, c(d.fg), c(d.bg)));
            }
        }
        grid
    }
}
