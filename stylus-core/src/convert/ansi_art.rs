//! ANSI-art by Weiran Huang (github.com/EtoDemerzel0427/ANSI-art,
//! Apache-2.0), ported from `art/solver.go`, `art/ansi.go` and
//! `cmd/image.go`.
//!
//! Its ANSI text mode: the characters of a sequence ("01" by default),
//! repeated across the whole image, each in its pixel's 24-bit color on
//! black, after a Lanczos resize with `imaging`.

use image::RgbaImage;

use super::grid::{Cell, Color, Grid};
use super::util::{imaging_lanczos, rows_for};
use super::{Converter, ConverterInfo};

pub struct AnsiArt;

static INFO: ConverterInfo = ConverterInfo {
    id: "ansi-art",
    name: "ANSI-art",
    origin: "github.com/EtoDemerzel0427/ANSI-art",
    copyright: "Copyright © 2021 Weiran Huang",
    license: "Apache-2.0",
    language: "Go",
    revision: "7fffe1168a82c649958f23b30ed3612f2df24982",
    settings: "ansi-art image -f <image> -W 80 -H <rows> (sequence \"01\", contrast 0, sharpening 0)",
    adaptations: "The original takes both sizes; the rows here keep the image's shape in 1:2 cells.",
    license_text: include_str!("licenses/ansi-art.txt"),
};

/// The `-s` default.
const SEQUENCE: &[u8] = b"01";

impl Converter for AnsiArt {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let rows = rows_for(image.width(), image.height(), columns, 0.5);
        // `TuneImage`: contrast 0 and sigma 0 leave the resized image as is.
        let img = imaging_lanczos(image, columns, rows);
        let mut grid = Grid::new(img.width() as usize, img.height() as usize);
        let mut seq = 0;
        for (x, y, p) in img.enumerate_pixels() {
            // `RGBA() >> 8` of an NRGBA pixel.
            let a = p[3] as u32 * 0x101;
            let c = |v: u8| ((v as u32 * 0x101 * a / 0xffff) >> 8) as u8;
            grid.set(x as usize, y as usize, Cell::new(SEQUENCE[seq], Color::Rgb(c(p[0]), c(p[1]), c(p[2])), Color::Rgb(0, 0, 0)));
            seq = (seq + 1) % SEQUENCE.len();
        }
        grid
    }
}
