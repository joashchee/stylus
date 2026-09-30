//! img2ansi by John McCabe (github.com/johnmccabe/img2ansi, MIT), ported
//! from `img2ansi.go` (`RenderANSI256`) and `palette/xterm256.go`.
//!
//! Half blocks in 256 colors: ▀ with the top pixel's nearest xterm color
//! as foreground and the bottom's as background (Go's
//! `color.Palette.Index`); transparent halves show the terminal's own.

use image::RgbaImage;

use super::grid::{cp437, Cell, Color, Grid};
use super::nfnt::GoImage;
use super::util::{go_palette256_index, resample, windows_xterm256, Kernel};
use super::{Converter, ConverterInfo};

pub struct Img2AnsiJohnmccabe;

static INFO: ConverterInfo = ConverterInfo {
    id: "img2ansi-johnmccabe",
    name: "img2ansi (John McCabe)",
    origin: "github.com/johnmccabe/img2ansi",
    copyright: "Copyright (c) 2017 John McCabe",
    license: "MIT",
    language: "Go",
    revision: "567be8c3332fcd2eaefcc2c5eae2c0db6d66c8f7",
    settings: "img2ansi.RenderANSI256 (the img2ansi command)",
    adaptations: "It draws one column per pixel of the image as given, with no resizing, so the image is first scaled to 80 pixels wide (area average).",
    license_text: include_str!("licenses/img2ansi-johnmccabe.txt"),
};

const ALPHA_THRESHOLD: u8 = 30;

impl Converter for Img2AnsiJohnmccabe {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let height = ((image.height() as f64 * columns as f64 / image.width() as f64).round() as u32).max(1);
        let img = GoImage::from_straight(&resample(image, columns, height, Kernel::Box));
        let rows = img.h.div_ceil(2);
        let mut grid = Grid::new(img.w as usize, rows as usize);
        for row in 0..rows {
            let y = row * 2;
            for x in 0..img.w {
                let top = img.rgba(x, y);
                // Past the last row, At() is transparent.
                let bottom = if y + 1 < img.h { img.rgba(x, y + 1) } else { [0; 4] };
                // uint8() of the 16-bit alpha keeps its low byte.
                let (at, ab) = ((top[3] & 0xff) as u8 >= ALPHA_THRESHOLD, (bottom[3] & 0xff) as u8 >= ALPHA_THRESHOLD);
                let cell = match (at, ab) {
                    (true, true) => Cell::new(cp437::UPPER_HALF, Color::Xterm(go_palette256_index(windows_xterm256, top)), Color::Xterm(go_palette256_index(windows_xterm256, bottom))),
                    (false, true) => Cell::new(cp437::LOWER_HALF, Color::Xterm(go_palette256_index(windows_xterm256, bottom)), Color::Default),
                    (true, false) => Cell::new(cp437::UPPER_HALF, Color::Xterm(go_palette256_index(windows_xterm256, top)), Color::Default),
                    (false, false) => Cell::BLANK,
                };
                grid.set(x as usize, row as usize, cell);
            }
        }
        grid
    }
}
