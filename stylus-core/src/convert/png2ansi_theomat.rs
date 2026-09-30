//! png2ansi by Théo Matricon (github.com/Theomat/png2ansi, Apache-2.0),
//! ported from `main.c`.
//!
//! Two spaces per pixel in the pixel's 24-bit color as the background, so
//! pixels come out square; pixels with alpha 127 or less are left
//! uncolored.

use image::RgbaImage;

use super::grid::{Cell, Color, Grid};
use super::util::{resample, Kernel};
use super::{Converter, ConverterInfo};

pub struct Png2AnsiTheomat;

static INFO: ConverterInfo = ConverterInfo {
    id: "png2ansi-theomat",
    name: "png2ansi (Théo Matricon)",
    origin: "github.com/Theomat/png2ansi",
    copyright: "Copyright Théo Matricon",
    license: "Apache-2.0",
    language: "C",
    revision: "28500d037481d2e6a557f7b33dce5c38b51536aa",
    settings: "png2ansi <png_file> (alpha threshold 127)",
    adaptations: "The original draws every pixel of the image as given, with no resizing, so the image is first scaled to 40 pixels wide (area average), which it draws 80 columns wide. It reads PNGs only; here any image Stylus decodes.",
    license_text: include_str!("licenses/png2ansi-theomat.txt"),
};

const ALPHA_THRESHOLD: u8 = 127;

impl Converter for Png2AnsiTheomat {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let pixels = (columns / 2).max(1);
        let height = ((pixels as f64 * image.height() as f64 / image.width() as f64).round() as u32).max(1);
        let img = resample(image, pixels, height, Kernel::Box);
        let mut grid = Grid::new(2 * img.width() as usize, img.height() as usize);
        for (x, y, p) in img.enumerate_pixels() {
            let bg = if p[3] > ALPHA_THRESHOLD { Color::Rgb(p[0], p[1], p[2]) } else { Color::Default };
            for dx in 0..2 {
                grid.set(2 * x as usize + dx, y as usize, Cell::new(b' ', Color::Default, bg));
            }
        }
        grid
    }
}
