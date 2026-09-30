//! ransid by mikefc (github.com/coolbutuseless/ransid, MIT), ported from
//! `R/image.R` (`im2char`) and `R/ansi-escape-codes.R` (`col2code`), with
//! the ImageMagick resize it runs through the magick package.
//!
//! One colored space per pixel, in xterm's 256 colors: each channel
//! rounded to the 6-level cube, or a pure gray to the 24-step gray ramp.

use image::RgbaImage;

use super::grid::{Cell, Color, Grid};
use super::magick;
use super::{Converter, ConverterInfo};

pub struct Ransid;

static INFO: ConverterInfo = ConverterInfo {
    id: "ransid",
    name: "ransid",
    origin: "github.com/coolbutuseless/ransid",
    copyright: "Copyright (c) 2020 mikefc",
    license: "MIT",
    language: "R",
    revision: "6910453277b02b882af3a2782bfc1c7332e32325",
    settings: "im2ansi(image_read(<image>)): width 80, font_aspect 0.45, 256 colors, with ImageMagick 7 (Q16 HDRI)",
    adaptations: "None beyond the shared width. As in the original, fully transparent pixels come out white and partly transparent ones keep their color. ImageMagick resizes an image with an alpha channel differently; here that means any image with a pixel that isn't opaque.",
    license_text: include_str!("licenses/ransid.txt"),
};

const FONT_ASPECT: f64 = 0.45;

/// `col2code`.
fn col2code(r: u8, g: u8, b: u8) -> u8 {
    // R's round: halves to even.
    let round = |v: f64| v.round_ties_even();
    if r == g && g == b && r != 255 {
        return 232 + round(r as f64 / 255.0 * 23.0) as u8;
    }
    let c = |v: u8| round(v as f64 / 255.0 * 5.0) as u8;
    16 + 36 * c(r) + 6 * c(g) + c(b)
}

impl Converter for Ransid {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        // `im2char`: the height for the width and font aspect.
        let width_factor = columns as f64 / image.width() as f64;
        let height = ((image.height() as f64 * width_factor * FONT_ASPECT).round_ties_even() as u32).max(1);
        let alpha = image.pixels().any(|p| p[3] < 255);
        let img = magick::resize(image, columns, height, alpha);
        let mut grid = Grid::new(img.width() as usize, img.height() as usize);
        for (x, y, p) in img.enumerate_pixels() {
            // `as.raster` names a fully transparent pixel "transparent",
            // which `col2rgb` reads as white.
            let code = if p[3] == 0 { col2code(255, 255, 255) } else { col2code(p[0], p[1], p[2]) };
            grid.set(x as usize, y as usize, Cell::new(b' ', Color::Default, Color::Xterm(code)));
        }
        grid
    }
}
