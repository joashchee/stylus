//! img2ansi by Bryan Matsuo (github.com/bmatsuo/img2ansi, MIT), ported
//! from `img2ansi.go` (`writeANSIPixels`), `ansipalette.go`
//! (`Palette256Precise`), `color256.go` and `resize.go`.
//!
//! One pixel per cell: a space whose background is the nearest of the 256
//! xterm colors (Go's `color.Palette.Index`), after a nearest-neighbor
//! resize for half-height cells. Any transparency leaves the cell blank.

use image::RgbaImage;

use super::grid::{Cell, Color, Grid};
use super::nfnt::{resize, GoImage, Interp};
use super::util::{go_palette256_index, windows_xterm256};
use super::{Converter, ConverterInfo};

pub struct Img2AnsiBmatsuo;

static INFO: ConverterInfo = ConverterInfo {
    id: "img2ansi-bmatsuo",
    name: "img2ansi (Bryan Matsuo)",
    origin: "github.com/bmatsuo/img2ansi",
    copyright: "Copyright (c) 2014 Bryan Matsuo",
    license: "MIT",
    language: "Go",
    revision: "09175604066e9af1d456763e414c2ea59f4cc154",
    settings: "img2ansi -width 80 -pad '' (256 colors, font aspect 0.5, alpha threshold 1.0)",
    adaptations: "None beyond the shared width.",
    license_text: include_str!("licenses/img2ansi-bmatsuo.txt"),
};

const FONT_ASPECT: f64 = 0.5;

/// `color256.go`: the Windows 16 system colors and xterm's cube and grays,
/// except two grays it lists as 0x60 and 0x66 (xterm's are 0x62 and 0x6c).
fn palette256(i: u8) -> [u8; 3] {
    match i {
        241 => [0x60; 3],
        242 => [0x66; 3],
        _ => windows_xterm256(i),
    }
}

/// Its `round`: floor(x + 0.5).
fn round(x: f64) -> f64 {
    (x + 0.5).floor()
}

impl Converter for Img2AnsiBmatsuo {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        // sizeRect(size, 80, 0, fontAspect): sizeNormal, then _sizeWidth.
        let (w, h) = (image.width() as f64, image.height() as f64);
        let norm_x = round(h * (w / h) / FONT_ASPECT);
        let aspect = norm_x / h;
        let height = (round(columns as f64 / aspect) as u32).max(1);
        let img = resize(columns, height, &GoImage::from_straight(image), Interp::NearestNeighbor);
        let mut grid = Grid::new(img.w as usize, img.h as usize);
        for y in 0..img.h {
            for x in 0..img.w {
                let c = img.rgba(x, y);
                // AlphaThreshold 0xffff: anything not fully opaque is cleared.
                if c[3] >= 0xffff {
                    grid.set(x as usize, y as usize, Cell::new(b' ', Color::Default, Color::Xterm(go_palette256_index(palette256, c))));
                }
            }
        }
        grid
    }
}
