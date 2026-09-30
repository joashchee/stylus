//! Rich Pixels by Darren Burns (github.com/darrenburns/rich-pixels, MIT),
//! ported from `rich_pixels/_renderer.py` (`HalfcellRenderer`).
//!
//! Half blocks in 24-bit color after a nearest-neighbor resize: ▄ in the
//! lower pixel's color over the upper one's. Fully transparent pixels are
//! left uncolored.

use image::{Rgba, RgbaImage};

use super::grid::{cp437, Cell, Color, Grid};
use super::util::pillow_nearest;
use super::{Converter, ConverterInfo};

pub struct RichPixels;

static INFO: ConverterInfo = ConverterInfo {
    id: "rich-pixels",
    name: "Rich Pixels",
    origin: "github.com/darrenburns/rich-pixels",
    copyright: "Copyright (c) 2023 Darren Burns",
    license: "MIT",
    language: "Python",
    revision: "a0745ebcc26b966d9dbac5875720364ee5c6a1d3",
    settings: "Pixels.from_image_path(<image>, resize=(80, <pixel rows>)) with the default HalfcellRenderer, printed by a 24-bit-color Rich console",
    adaptations: "The original takes the size in pixels; the pixel rows here keep the image's shape (two square pixels per 1:2 cell).",
    license_text: include_str!("licenses/rich-pixels.txt"),
};

/// `_get_color`: a pixel's color, or none when it's fully transparent.
fn color(p: &Rgba<u8>) -> Option<Color> {
    (p[3] > 0).then_some(Color::Rgb(p[0], p[1], p[2]))
}

impl Converter for RichPixels {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let rows = (columns as f64 * image.height() as f64 / image.width() as f64).round().max(1.0) as u32;
        // `HalfcellRenderer.render` makes the height even.
        let img = pillow_nearest(image, columns, rows + rows % 2);
        let mut grid = Grid::new(img.width() as usize, img.height() as usize / 2);
        for row in 0..grid.height {
            for x in 0..grid.width {
                let upper = color(img.get_pixel(x as u32, 2 * row as u32));
                let lower = color(img.get_pixel(x as u32, 2 * row as u32 + 1));
                let cell = match lower {
                    Some(fg) => Cell::new(cp437::LOWER_HALF, fg, upper.unwrap_or(Color::Default)),
                    None => Cell::new(b' ', Color::Default, upper.unwrap_or(Color::Default)),
                };
                grid.set(x, row, cell);
            }
        }
        grid
    }
}
