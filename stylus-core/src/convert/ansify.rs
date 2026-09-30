//! ansify by widberg (github.com/widberg/ansify, MIT), ported from
//! `src/lib.rs` and `ansify-cli/src/main.rs`, with its `res/16.yaml`
//! palette and `res/classic.yaml` blocks.
//!
//! Every pixel becomes one character. Each shade glyph (░ ▒ ▓ █) covers a
//! fraction of its 7×17 cell, so every foreground/background pair of the
//! palette blended by that fraction is a color it can show; each pixel
//! takes the nearest one.

use image::imageops::FilterType;
use image::RgbaImage;

use super::grid::{cp437, Cell, Color, Grid};
use super::{Converter, ConverterInfo};

pub struct Ansify;

static INFO: ConverterInfo = ConverterInfo {
    id: "ansify",
    name: "ansify",
    origin: "github.com/widberg/ansify",
    copyright: "Copyright (c) 2022 widberg",
    license: "MIT",
    language: "Rust",
    revision: "3dc3f9e94d414b10d044877935ebc9f7039135c5",
    settings: "ansify-cli -p res/16.yaml -b res/classic.yaml -w 80 image --text",
    adaptations: "Nearest colors are found by a linear scan instead of its k-d tree, so an exact tie can pick a different, equally near, pair.",
    license_text: include_str!("licenses/ansify.txt"),
};

/// res/16.yaml. Written as xterm color numbers (`ansi_term`'s `Fixed`).
const PALETTE: [[u8; 3]; 16] = [
    [30, 30, 30],
    [255, 0, 0],
    [0, 255, 0],
    [255, 255, 0],
    [0, 0, 255],
    [255, 0, 255],
    [0, 255, 255],
    [204, 204, 204],
    [102, 102, 102],
    [241, 76, 76],
    [35, 209, 139],
    [245, 245, 67],
    [59, 142, 234],
    [214, 112, 214],
    [41, 184, 219],
    [229, 229, 229],
];

/// res/classic.yaml's blocks in its BTreeMap order (by code point: █, ░,
/// ▒, ▓), with the share of their 7×17 bitmaps in the foreground.
const BLOCKS: [(u8, u32); 4] = [(cp437::FULL_BLOCK, 119), (cp437::LIGHT_SHADE, 32), (cp437::MEDIUM_SHADE, 60), (cp437::DARK_SHADE, 88)];
const BLOCK_W: u32 = 7;
const BLOCK_H: u32 = 17;

fn normalize(c: [u8; 3]) -> [f32; 3] {
    c.map(|v| v as f32 / 255.0)
}

/// (color, fg, bg, glyph), in `ANSIfier::new`'s order.
fn texels() -> Vec<([f32; 3], u8, u8, u8)> {
    let mut out = Vec::new();
    for &(glyph, count) in &BLOCKS {
        let ratio = count as f32 / (BLOCK_W * BLOCK_H) as f32;
        if ratio == 0.0 {
            for (i, &c) in PALETTE.iter().enumerate() {
                out.push((normalize(c), 0, i as u8, glyph));
            }
        } else if ratio == 1.0 {
            for (i, &c) in PALETTE.iter().enumerate() {
                out.push((normalize(c), i as u8, 0, glyph));
            }
        } else {
            for (i, &f) in PALETTE.iter().enumerate() {
                for (j, &b) in PALETTE.iter().enumerate() {
                    if f == b {
                        continue;
                    }
                    let (a, b) = (normalize(f), normalize(b));
                    let blend = [a[0] * ratio + b[0] * (1.0 - ratio), a[1] * ratio + b[1] * (1.0 - ratio), a[2] * ratio + b[2] * (1.0 - ratio)];
                    out.push((blend, i as u8, j as u8, glyph));
                }
            }
        }
    }
    out
}

impl Converter for Ansify {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        // calculate_new_dimensions with only a width.
        let ratio = (image.width() as f32 / BLOCK_W as f32) / (image.height() as f32 / BLOCK_H as f32);
        let rows = ((columns as f32 / ratio) as u32).max(1);
        let img = image::imageops::resize(image, columns, rows, FilterType::Lanczos3);
        let texels = texels();
        let mut grid = Grid::new(columns as usize, rows as usize);
        for (x, y, p) in img.enumerate_pixels() {
            let q = [p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0];
            let mut best = &texels[0];
            let mut best_d = f32::MAX;
            for t in &texels {
                let d = (t.0[0] - q[0]).powi(2) + (t.0[1] - q[1]).powi(2) + (t.0[2] - q[2]).powi(2);
                if d < best_d {
                    best_d = d;
                    best = t;
                }
            }
            grid.set(x as usize, y as usize, Cell::new(best.3, Color::Xterm(best.1), Color::Xterm(best.2)));
        }
        grid
    }
}
