//! img2ansi by Lukas Beranek (github.com/lloiser/img2ansi, MIT), ported
//! from `img2ansi.go` and `cmd/img2ansi/main.go`.
//!
//! A near copy of ansize: every pixel becomes a 0 in its color from
//! xterm's 6×6×6 cube, black included.

use image::RgbaImage;

use super::grid::{Cell, Color, Grid};
use super::nfnt::{resize, GoImage, Interp};
use super::{Converter, ConverterInfo};

pub struct Img2AnsiLloiser;

static INFO: ConverterInfo = ConverterInfo {
    id: "img2ansi-lloiser",
    name: "img2ansi (Lukas Beranek)",
    origin: "github.com/lloiser/img2ansi",
    copyright: "Copyright (c) 2017 Lukas Beranek",
    license: "MIT",
    language: "Go",
    revision: "1b5600c638db787130d33940b4ea4349dd5517d5",
    settings: "img2ansi -w 80 <image>",
    adaptations: "The original never enlarges (an image narrower than 80 pixels keeps its width); here every image is scaled to 80 columns.",
    license_text: include_str!("licenses/img2ansi-lloiser.txt"),
};

/// The width/height ratio of a typical fixed-width font, its `ratio`.
const RATIO: f64 = 0.5;

/// `toAnsiSpace`: a 16-bit channel to 0–5.
fn space(v: u32) -> u32 {
    (6.0 * (v as f64 / 65536.0)) as u32
}

impl Converter for Img2AnsiLloiser {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let cur_ratio = image.height() as f64 / image.width() as f64;
        let height = ((columns as f64 * cur_ratio * RATIO) as u32).max(1);
        let img = resize(columns, height, &GoImage::from_straight(image), Interp::Lanczos3);
        let mut grid = Grid::new(img.w as usize, img.h as usize);
        for y in 0..img.h {
            for x in 0..img.w {
                let [r, g, b, _] = img.rgba(x, y);
                let code = 16 + space(r) * 36 + space(g) * 6 + space(b);
                grid.set(x as usize, y as usize, Cell::new(b'0', Color::Xterm(code as u8), Color::Default));
            }
        }
        grid
    }
}
