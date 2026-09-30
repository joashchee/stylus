//! tapciify by Aleksei Rybin (github.com/tapciify/tapciify, MIT), ported
//! from `src/renderers/ascii.rs`, `src/utils/resize.rs` and
//! `src/utils/player.rs`.
//!
//! Colored ASCII: one character per pixel from the ramp ` .,:;+*?%S#@`
//! by luma times alpha, in the pixel's 24-bit color. It resizes and takes
//! luma with the `image` crate, which Stylus uses too, so those parts are
//! the same code.

use image::imageops::FilterType;
use image::{DynamicImage, Pixel, RgbaImage};

use super::grid::{Cell, Color, Grid};
use super::{Converter, ConverterInfo};

pub struct Tapciify;

static INFO: ConverterInfo = ConverterInfo {
    id: "tapciify",
    name: "tapciify",
    origin: "github.com/tapciify/tapciify",
    copyright: "Copyright (c) 2024 Aleksei Rybin",
    license: "MIT",
    language: "Rust",
    revision: "863d30e988a363f6bb613d3c22ecfd85d2269a14",
    settings: "tapciify -i <image> -w 80 -c",
    adaptations: "None.",
    license_text: include_str!("licenses/tapciify.txt"),
};

/// `DEFAULT_ASCII_STRING`.
const ASCII_STRING: &[u8] = b" .,:;+*?%S#@";
/// `DEFAULT_FONT_RATIO`, Consolas.
const FONT_RATIO: f64 = 11.0 / 24.0;

impl Converter for Tapciify {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        // `calc_new_height`.
        let height = (columns as f64 * FONT_RATIO * image.height() as f64 / image.width() as f64) as u32;
        // Decoded as the original would have it: RGB for an opaque image.
        let img = if image.pixels().all(|p| p[3] == 255) { DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(image.clone()).to_rgb8()) } else { DynamicImage::ImageRgba8(image.clone()) };
        let img = img.resize_exact(columns, height.max(1), FilterType::Triangle).to_rgba8();
        let mut grid = Grid::new(img.width() as usize, img.height() as usize);
        for (x, y, p) in img.enumerate_pixels() {
            let la = p.to_luma_alpha();
            let lightness = la[0] as f32 * la[1] as f32 / (255.0 * 255.0);
            let ch = ASCII_STRING[((ASCII_STRING.len() - 1) as f32 * lightness) as usize];
            grid.set(x as usize, y as usize, Cell::new(ch, Color::Rgb(p[0], p[1], p[2]), Color::Default));
        }
        grid
    }
}
