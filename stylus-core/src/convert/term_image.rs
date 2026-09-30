//! term-image by Toluwaleke Ogundipe (github.com/AnonymouX47/term-image,
//! MIT), ported from `src/term_image/image/block.py` and `common.py`
//! (`BlockImage`).
//!
//! Half blocks in 24-bit color after an area-average resize: ▀ in the
//! upper pixel's color over the lower one's, or a space where the two
//! match. Pixels under its alpha threshold (40/255) are left uncolored;
//! the rest are blended over black.

use image::RgbaImage;

use super::grid::{cp437, Cell, Color, Grid};
use super::util::{pillow_over, resample, Kernel};
use super::{Converter, ConverterInfo};

pub struct TermImage;

static INFO: ConverterInfo = ConverterInfo {
    id: "term-image",
    name: "term-image",
    origin: "github.com/AnonymouX47/term-image",
    copyright: "Copyright (c) 2022 Toluwaleke Ogundipe",
    license: "MIT",
    language: "Python",
    revision: "a40a12bb0a12944b7d7a182e93f1caad4261693a",
    settings: "BlockImage.from_file(<image>, width=80), default alpha threshold, cell ratio 0.5 and black terminal background",
    adaptations: "None.",
    license_text: include_str!("licenses/term-image.txt"),
};

/// `_ALPHA_THRESHOLD`, 40/255, as `round(alpha * 255)`.
const ALPHA_THRESHOLD: u8 = 40;

impl Converter for TermImage {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        // `_valid_size` for a width in columns, pixel ratio 1 (cell ratio
        // 0.5, two pixels a cell), then `_pixels_lines`: Python's round and
        // ceil.
        let height_px = (columns as f64 / image.width() as f64 * image.height() as f64).round_ties_even();
        let lines = ((height_px / 2.0).ceil() as u32).max(1);
        let img = resample(image, columns, lines * 2, Kernel::Box);
        // An RGB image has no transparency to check.
        let alpha = image.pixels().any(|p| p[3] < 255);
        let px = |x: u32, y: u32| {
            let p = *img.get_pixel(x, y);
            let [r, g, b] = pillow_over(p.0, [0, 0, 0]);
            (Color::Rgb(r, g, b), !alpha || p[3] >= ALPHA_THRESHOLD)
        };
        let mut grid = Grid::new(columns as usize, lines as usize);
        for row in 0..lines {
            for x in 0..columns {
                let (upper, upper_shows) = px(x, 2 * row);
                let (lower, lower_shows) = px(x, 2 * row + 1);
                // `update_buffer`.
                let cell = match (upper_shows, lower_shows) {
                    (false, false) => Cell::BLANK,
                    (false, true) => Cell::new(cp437::LOWER_HALF, lower, Color::Default),
                    (true, false) => Cell::new(cp437::UPPER_HALF, upper, Color::Default),
                    (true, true) if upper == lower => Cell::new(b' ', Color::Default, lower),
                    (true, true) => Cell::new(cp437::UPPER_HALF, upper, lower),
                };
                grid.set(x as usize, row as usize, cell);
            }
        }
        grid
    }
}
