//! png2ansi by Erich Erstu (github.com/1Hyena/png2ansi, MIT), ported from
//! `src/main.cpp`.
//!
//! Every pixel becomes one character. It builds a table of every color
//! the 8 ANSI colors can show as a shade glyph (█ ▓ ▒ ░ or space) of one
//! foreground, normal or bold, over one background, and each pixel takes
//! the nearest entry.

use std::collections::BTreeMap;

use image::RgbaImage;

use super::grid::{cp437, Cell, Color, Grid};
use super::util::{resample, Kernel};
use super::{Converter, ConverterInfo};

pub struct Png2Ansi1Hyena;

static INFO: ConverterInfo = ConverterInfo {
    id: "png2ansi-1hyena",
    name: "png2ansi (1Hyena)",
    origin: "github.com/1Hyena/png2ansi",
    copyright: "Copyright (c) 2017 Erich Erstu",
    license: "MIT",
    language: "C++",
    revision: "82b80e5146552e5c44aa934081f0b61f058b15cd",
    settings: "shade glyphs over 8 backgrounds, normal and bold foregrounds",
    adaptations: "The original maps each pixel of the image as given to one character, with no resizing, so the image is first scaled to 80 columns (area average, rows halved for the 1:2 cells).",
    license_text: include_str!("licenses/png2ansi-1hyena.txt"),
};

/// `color_table` and `bright_color_table`, SGR order.
const NORMAL: [[u8; 3]; 8] = [[0, 0, 0], [128, 0, 0], [0, 128, 0], [128, 128, 0], [0, 0, 128], [128, 0, 128], [0, 128, 128], [192, 192, 192]];
const BRIGHT: [[u8; 3]; 8] = [[128, 128, 128], [255, 0, 0], [0, 255, 0], [255, 255, 0], [0, 0, 255], [255, 0, 255], [0, 255, 255], [255, 255, 255]];

/// Shades by index: full block, dark, medium, light, none.
const SHADES: [u8; 5] = [cp437::FULL_BLOCK, cp437::DARK_SHADE, cp437::MEDIUM_SHADE, cp437::LIGHT_SHADE, b' '];

/// `ansi2rgb`: the color a shade glyph shows.
fn mix(bright: bool, fg: usize, bg: usize, shade: usize) -> [u8; 3] {
    let opacity = [1.0, 0.75, 0.5, 0.25, 0.0][shade];
    let f = if bright { BRIGHT[fg] } else { NORMAL[fg] };
    let b = NORMAL[bg];
    // (unsigned char) of a double truncates.
    std::array::from_fn(|c| (opacity * f[c] as f64 + (1.0 - opacity) * b[c] as f64) as u8)
}

/// The palette map: key r + 256·g + 65536·b (so iteration is in that
/// order), value the cell. Later entries replace earlier ones with the
/// same color, as `palette[index] = buf` does.
fn palette() -> BTreeMap<u32, Cell> {
    let mut map = BTreeMap::new();
    for bright in [true, false] {
        for bg in 0..8 {
            for fg in 0..8 {
                for shade in 0..5 {
                    let [r, g, b] = mix(bright, fg, bg, shade);
                    let fg_color = Color::Ansi(fg as u8 + if bright { 8 } else { 0 });
                    map.insert(r as u32 + 256 * g as u32 + 65536 * b as u32, Cell::new(SHADES[shade], fg_color, Color::Ansi(bg as u8)));
                }
            }
        }
    }
    map
}

impl Converter for Png2Ansi1Hyena {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let rows = super::util::rows_for(image.width(), image.height(), columns, 0.5);
        let img = resample(image, columns, rows, Kernel::Box);
        let palette = palette();
        let mut grid = Grid::new(columns as usize, rows as usize);
        for (x, y, p) in img.enumerate_pixels() {
            let cell = if p[3] < 128 {
                Cell::BLANK
            } else {
                // color_index: the first (lowest key) of the nearest.
                let mut best = Cell::BLANK;
                let mut best_d = u64::MAX;
                for (&key, &cell) in &palette {
                    let (r, g, b) = (key % 256, (key / 256) % 256, key / 65536);
                    let d = |a: u8, b: u32| (a as i64 - b as i64).unsigned_abs();
                    let dist = d(p[0], r).pow(2) + d(p[1], g).pow(2) + d(p[2], b).pow(2);
                    if dist < best_d {
                        best_d = dist;
                        best = cell;
                    }
                }
                best
            };
            grid.set(x as usize, y as usize, cell);
        }
        grid
    }
}
