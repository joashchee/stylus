//! imgcat by Eddie Antonio Santos (github.com/eddieantonio/imgcat, ISC),
//! ported from `src/print_image.c` (`half_height_image_iterator`,
//! `printer_true_color`) and `src/load_image.cc` (`maybe_resize`, with
//! CImg's nearest-neighbor resize).
//!
//! Half blocks in 24-bit color: ▀ with the top pixel as foreground and the
//! bottom as background, after a nearest-neighbor shrink. Transparency is
//! ignored: pixels keep the color stored under it.

use image::RgbaImage;

use super::grid::{cp437, Cell, Color, Grid};
use super::{Converter, ConverterInfo};

pub struct EddieantonioImgcat;

static INFO: ConverterInfo = ConverterInfo {
    id: "imgcat-eddieantonio",
    name: "imgcat (Eddie Antonio Santos)",
    origin: "github.com/eddieantonio/imgcat",
    copyright: "Copyright (c) 2014-2019 Eddie Antonio Santos",
    license: "ISC",
    language: "C",
    revision: "b7464bae104d1a106b652863b719eed9367cb8e9",
    settings: "imgcat --depth=24bit --half-height in an 80-column terminal",
    adaptations: "It only shrinks, so an image narrower than 80 pixels is enlarged to 80 columns here (the same nearest-neighbor resize).",
    license_text: include_str!("licenses/imgcat-eddieantonio.txt"),
};

impl Converter for EddieantonioImgcat {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let (w, h) = image.dimensions();
        // maybe_resize: width to the terminal's, height by the ratio.
        let (nw, nh) = if w == columns {
            (w, h)
        } else {
            let ratio = h as f64 / w as f64;
            (columns, ((ratio * columns as f64) as u32).max(1))
        };
        // CImg resize type 1: source index floor(i · old / new).
        let px = |x: u32, y: u32| image.get_pixel((x as u64 * w as u64 / nw as u64) as u32, (y as u64 * h as u64 / nh as u64) as u32);
        let rows = (nh / 2).max(1);
        let mut grid = Grid::new(nw as usize, rows as usize);
        for row in 0..nh / 2 {
            for x in 0..nw {
                let (t, b) = (px(x, row * 2), px(x, row * 2 + 1));
                grid.set(x as usize, row as usize, Cell::new(cp437::UPPER_HALF, Color::Rgb(t[0], t[1], t[2]), Color::Rgb(b[0], b[1], b[2])));
            }
        }
        grid
    }
}
