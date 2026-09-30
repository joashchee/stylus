//! img2txt by hit9 (github.com/hit9/img2txt, BSD-3-Clause), ported from
//! `img2txt.py` (`load_and_resize_image`) and `ansi.py`
//! (`generate_ANSI_from_pixels`).
//!
//! One pixel per cell: a space whose background is the pixel's color
//! rounded to xterm's 6×6×6 color cube.

use image::RgbaImage;

use super::grid::{Cell, Color, Grid};
use super::util::{resample, Kernel};
use super::{Converter, ConverterInfo};

pub struct Img2txt;

static INFO: ConverterInfo = ConverterInfo {
    id: "img2txt-hit9",
    name: "img2txt (hit9)",
    origin: "github.com/hit9/img2txt",
    copyright: "Copyright (c) 2013 - 2016, hit9",
    license: "BSD-3-Clause",
    language: "Python",
    revision: "b54bf0cc9ac274a7fa738e06b534ec7975ad9c18",
    settings: "img2txt.py --ansi --antialias --targetAspect=0.5 --maxLen (the larger side, set so the width is 80)",
    adaptations: "Its --antialias names Pillow's ANTIALIAS filter, which Pillow 10 removed; it's the Lanczos filter it was an alias for.",
    license_text: include_str!("licenses/img2txt-hit9.txt"),
};

/// `getANSIcolor_for_rgb`: each channel rounded to the cube's 6 levels.
fn cube(r: u8, g: u8, b: u8) -> u8 {
    let level = |v: u8| ((v as f64 / 255.0) * 5.0).round() as u8;
    16 + level(r) * 36 + level(g) * 6 + level(b)
}

impl Converter for Img2txt {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        // load_and_resize_image: the height scaled by the aspect first, then
        // both by maxLen over the larger side (int() truncates).
        let (w, h) = (image.width() as f64, image.height() as f64);
        let aspect_h = (0.5 * h).trunc();
        let max_len = if w >= aspect_h { columns as f64 } else { columns as f64 * aspect_h / w };
        let rate = max_len / w.max(aspect_h);
        let new_w = ((rate * w) as u32).max(1);
        let new_h = ((rate * aspect_h) as u32).max(1);
        let img = resample(image, new_w, new_h, Kernel::Lanczos3);
        let mut grid = Grid::new(new_w as usize, new_h as usize);
        for (x, y, p) in img.enumerate_pixels() {
            // A fully transparent pixel isn't drawn.
            if p[3] != 0 {
                grid.set(x as usize, y as usize, Cell::new(b' ', Color::Default, Color::Xterm(cube(p[0], p[1], p[2]))));
            }
        }
        grid
    }
}
