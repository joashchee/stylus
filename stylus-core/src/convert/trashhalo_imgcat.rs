//! imgcat by Stephen Solka (github.com/trashhalo/imgcat, MIT), ported
//! from `component/load.go` (`imageToString`), with go-colorful's
//! `MakeColor` and termenv's true-color output.
//!
//! Half blocks in 24-bit color: ▀ with the top pixel as foreground and the
//! bottom as background, after a Lanczos3 thumbnail.

use image::RgbaImage;

use super::grid::{cp437, Cell, Color, Grid};
use super::nfnt::{resize, thumbnail, GoImage, Interp};
use super::{Converter, ConverterInfo};

pub struct TrashhaloImgcat;

static INFO: ConverterInfo = ConverterInfo {
    id: "imgcat-trashhalo",
    name: "imgcat (Stephen Solka)",
    origin: "github.com/trashhalo/imgcat",
    copyright: "Copyright (c) 2020 Stephen Solka",
    license: "MIT",
    language: "Go",
    revision: "8ed8c30ab6197abdc6a4ff0049f9663d4950d5e9",
    settings: "imgcat in an 80-column true-color terminal tall enough for the whole image",
    adaptations: "Its thumbnail only shrinks, so an image narrower than 80 pixels is enlarged to 80 columns here (same filter).",
    license_text: include_str!("licenses/imgcat-trashhalo.txt"),
};

/// go-colorful's `MakeColor(c).Hex()` for a pixel's premultiplied
/// 16-bit color (un-premultiplied, rounded to 8 bits), as termenv's
/// `RGBColor.Sequence` writes it.
fn color(c: Option<[u32; 4]>) -> Color {
    let [r, g, b, a] = c.unwrap_or([0; 4]);
    if a == 0 {
        return Color::Rgb(0, 0, 0);
    }
    let un = |v: u32| {
        let v = v * 0xffff / a;
        let hex = ((v as f64 / 65535.0) * 255.0 + 0.5) as u8;
        // termenv parses the hex back (×1/255) and truncates ×255, which
        // can lose one.
        (hex as f64 * (1.0 / 255.0) * 255.0) as u8
    };
    Color::Rgb(un(r), un(g), un(b))
}

impl Converter for TrashhaloImgcat {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let src = GoImage::from_straight(image);
        let img = if image.width() < columns {
            let h = ((image.height() as u64 * columns as u64 / image.width() as u64) as u32).max(1);
            resize(columns, h, &src, Interp::Lanczos3)
        } else {
            thumbnail(columns, u32::MAX, &src, Interp::Lanczos3)
        };
        let rows = img.h.div_ceil(2);
        let mut grid = Grid::new(img.w as usize, rows as usize);
        for row in 0..rows {
            let y = row * 2;
            for x in 0..img.w {
                // Past the last row, At() is transparent black.
                let bottom = (y + 1 < img.h).then(|| img.rgba(x, y + 1));
                grid.set(x as usize, row as usize, Cell::new(cp437::UPPER_HALF, color(Some(img.rgba(x, y))), color(bottom)));
            }
        }
        grid
    }
}
