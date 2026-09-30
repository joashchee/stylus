//! asciimatics by Peter Brittain (github.com/peterbrittain/asciimatics,
//! Apache-2.0), ported from `asciimatics/renderers/images.py`
//! (`ColourImageFile`) and `asciimatics/screen.py` (its 256-color
//! palette), with the Pillow (MIT-CMU) conversion it quantizes through.
//!
//! Half blocks in xterm's 256 colors: a bicubic resize, then each pixel to
//! the nearest palette color, ▄ in the lower pixel's over the upper
//! one's. Where both are mostly transparent, a black-on-black dot.

use image::RgbaImage;

use super::grid::{cp437, Cell, Color, Grid};
use super::util::{drop_alpha, resample, windows_xterm256, Kernel};
use super::{Converter, ConverterInfo};

pub struct Asciimatics;

static INFO: ConverterInfo = ConverterInfo {
    id: "asciimatics",
    name: "asciimatics",
    origin: "github.com/peterbrittain/asciimatics",
    copyright: "Copyright 2016 Peter Brittain",
    license: "Apache-2.0",
    language: "Python",
    revision: "bf0cea87b50439e40a5c7b708d64da4195314a60",
    settings: "ColourImageFile(screen, <image>, height=<rows>, uni=True) on a 256-color screen (bg black, no dithering, no background fill)",
    adaptations: "The original sizes by height; the rows here are the most that keep the width within 80 columns (and an image over 80 times wider than tall is cut to 80). It ends every row with a black-on-black dot to reset terminals, which would make an 81st column, so that's left out.",
    license_text: include_str!("licenses/asciimatics.txt"),
};

/// `Screen.COLOUR_BLACK`, the default `bg`.
const BG: u8 = 0;

/// Pillow's RGB→P conversion without dithering (`topalette` with
/// `ImagingPaletteCacheUpdate`): a color is looked up by its top 6 bits
/// per channel, and each such slot holds the palette entry nearest its
/// low corner (squared RGB distance, the first entry on a tie).
fn quantize([r, g, b]: [u8; 3]) -> u8 {
    let slot = [r & 0xfc, g & 0xfc, b & 0xfc].map(|v| v as i32);
    let mut best = 0;
    let mut best_d = u32::MAX;
    for i in 0..=255u8 {
        let p = windows_xterm256(i).map(|v| v as i32);
        let d = (0..3).map(|c| ((slot[c] - p[c]) * (slot[c] - p[c])) as u32).sum::<u32>();
        if d < best_d {
            best = i;
            best_d = d;
        }
    }
    best
}

impl Converter for Asciimatics {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let (w, h) = (image.width() as f64, image.height() as f64);
        let height = ((columns as f64 * h / (2.0 * w)) as u32).max(1);
        let width = ((w * height as f64 * 2.0 / h) as u32).clamp(1, columns);
        let frame = resample(image, width, height * 2, Kernel::Bicubic);
        // `new_frame2`: RGB (alpha dropped), then quantized.
        let rgb = drop_alpha(&frame);
        let col = |x: u32, y: u32| {
            let p = rgb.get_pixel(x, y);
            Color::Xterm(quantize([p[0], p[1], p[2]]))
        };
        let mut grid = Grid::new(width as usize, height as usize);
        for row in 0..height {
            let py = row * 2;
            for px in 0..width {
                let cell = if frame.get_pixel(px, py)[3] < 64 && frame.get_pixel(px, py + 1)[3] < 64 {
                    Cell::new(b'.', Color::Xterm(BG), Color::Xterm(BG))
                } else {
                    Cell::new(cp437::LOWER_HALF, col(px, py + 1), col(px, py))
                };
                grid.set(px as usize, row as usize, cell);
            }
        }
        grid
    }
}
