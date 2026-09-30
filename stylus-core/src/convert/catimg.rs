//! catimg by Eduardo San Martin Morote (github.com/posva/catimg, MIT),
//! ported from `src/catimg.c` and `img_resize` in `src/sh_image.c`.
//!
//! Half blocks in 24-bit color, after a box-average shrink: the top pixel
//! as the background and the bottom as the foreground of ▄, or ▀/▄ alone
//! when one of the two is transparent.

use image::{Rgba, RgbaImage};

use super::grid::{cp437, Cell, Color, Grid};
use super::{Converter, ConverterInfo};

pub struct Catimg;

static INFO: ConverterInfo = ConverterInfo {
    id: "catimg",
    name: "catimg",
    origin: "github.com/posva/catimg",
    copyright: "Copyright (c) 2013-2017 Eduardo San Martin Morote",
    license: "MIT",
    language: "C",
    revision: "7ae0fc7c8e46df112211bcc57337a8e6f2230869",
    settings: "catimg in an 80-column UTF-8 terminal: resolution 2 (half blocks), true color",
    adaptations: "catimg only shrinks (its box filter can't enlarge), so an image narrower than 80 pixels is enlarged to 80 columns here by repeating pixels.",
    license_text: include_str!("licenses/catimg.txt"),
};

/// Transparency threshold: alpha below 25%.
const TRANSP_ALPHA: u8 = 64;

/// `img_resize` with one scale for both axes, in 32-bit float as in C.
/// Each output pixel averages the whole-pixel block under it.
fn img_resize(img: &RgbaImage, scale: f32) -> RgbaImage {
    let w = ((img.width() as f32 * scale) as u32).max(1);
    let h = ((img.height() as f32 * scale) as u32).max(1);
    let inv = 1.0f32 / scale;
    // Enlarging makes the block 0 pixels (a division by zero in C): take
    // the one pixel under it instead.
    let block = (inv as i32).max(1) as u32;
    let wh = block * block;
    RgbaImage::from_fn(w, h, |x, y| {
        let sx = (x as f32 * inv) as u32;
        let sy = (y as f32 * inv) as u32;
        let mut sum = [0u32; 4];
        for yi in 0..block {
            for xi in 0..block {
                let p = img.get_pixel((sx + xi).min(img.width() - 1), (sy + yi).min(img.height() - 1));
                for c in 0..4 {
                    sum[c] += p[c] as u32;
                }
            }
        }
        Rgba(sum.map(|v| (v / wh) as u8))
    })
}

fn rgb(p: &Rgba<u8>) -> Color {
    Color::Rgb(p[0], p[1], p[2])
}

impl Converter for Catimg {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let resized;
        let img = if image.width() == columns {
            image
        } else {
            resized = img_resize(image, columns as f32 / image.width() as f32);
            &resized
        };
        let (w, h) = img.dimensions();
        let rows = h.div_ceil(2);
        let mut grid = Grid::new(w as usize, rows as usize);
        for row in 0..rows {
            let y = row * 2;
            for x in 0..w {
                let upper = img.get_pixel(x, y);
                let cell = if y < h - 1 {
                    let lower = img.get_pixel(x, y + 1);
                    match (upper[3] < TRANSP_ALPHA, lower[3] < TRANSP_ALPHA) {
                        (true, true) => Cell::BLANK,
                        (true, false) => Cell::new(cp437::LOWER_HALF, rgb(lower), Color::Default),
                        (false, true) => Cell::new(cp437::UPPER_HALF, rgb(upper), Color::Default),
                        (false, false) => Cell::new(cp437::LOWER_HALF, rgb(lower), rgb(upper)),
                    }
                } else if upper[3] < TRANSP_ALPHA {
                    Cell::BLANK
                } else {
                    Cell::new(cp437::UPPER_HALF, rgb(upper), Color::Default)
                };
                grid.set(x as usize, row as usize, cell);
            }
        }
        grid
    }
}
