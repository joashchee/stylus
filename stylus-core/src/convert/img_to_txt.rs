//! img_to_txt by Danny Burrows (github.com/danny-burrows/img_to_txt, MIT),
//! ported from `src/main.c` (`read_and_convert`, `calc_ascii_char`), with
//! the stb_image_resize2 resize it links (`stbir.rs`).
//!
//! Colored ASCII: one character per pixel from a 70-step ramp by luminance,
//! in the pixel's 24-bit color.

use image::RgbaImage;

use super::grid::{Cell, Color, Grid};
use super::stbir;
use super::util::rows_for;
use super::{Converter, ConverterInfo};

pub struct ImgToTxt;

static INFO: ConverterInfo = ConverterInfo {
    id: "img-to-txt",
    name: "img_to_txt",
    origin: "github.com/danny-burrows/img_to_txt",
    copyright: "Copyright (c) 2024 Danny Burrows",
    license: "MIT",
    language: "C",
    revision: "dd5e73409c6a892f1b9c36be45c624893b16e212",
    settings: "img_to_txt -q -w 80 -h <rows> <image> (colored ASCII)",
    adaptations: "The original takes both sizes, or sizes to the terminal; the rows here keep the image's shape with its default squashing (pixels twice as wide as tall). It reads images as RGB, so alpha is ignored.",
    license_text: include_str!("licenses/img-to-txt.txt"),
};

/// `scale`, dark to light; the last used entry is the space before the
/// string's terminating NUL.
const SCALE: &[u8] = b"$@&B%8WM#ZO0QoahkbdpqwmLCJUYXIjft/\\|()1{}[]l?zcvunxr!<>i;:*-+~_,\"^`'. ";
/// `brightness_levels`: the array's size (with its NUL) less 2.
const LEVELS: u32 = SCALE.len() as u32 + 1 - 2;

impl Converter for ImgToTxt {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let rows = rows_for(image.width(), image.height(), columns, 0.5);
        let px: Vec<[u8; 3]> = if (columns, rows) == image.dimensions() {
            image.pixels().map(|p| [p[0], p[1], p[2]]).collect()
        } else {
            stbir::resize_rgb(image, columns, rows)
        };
        let mut grid = Grid::new(columns as usize, rows as usize);
        for (i, &[r, g, b]) in px.iter().enumerate() {
            // `luminanceFromRGB` truncates to a byte.
            let lum = (0.2126 * r as f64 + 0.7152 * g as f64 + 0.0722 * b as f64) as u8 as u32;
            let ch = SCALE[(LEVELS - lum * LEVELS / 256) as usize];
            grid.set(i % columns as usize, i / columns as usize, Cell::new(ch, Color::Rgb(r, g, b), Color::Default));
        }
        grid
    }
}
