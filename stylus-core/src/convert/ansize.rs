//! ansize by Jason Chen (github.com/jhchen/ansize, MIT), ported from
//! `ansize.go`.
//!
//! "Binary ANSI art": every pixel becomes a 0 or a 1, picked at random,
//! in the pixel's color from xterm's 6×6×6 cube; black pixels are blank.

use image::RgbaImage;

use super::grid::{Cell, Color, Grid};
use super::nfnt::{resize, GoImage, Interp};
use super::util::SplitMix64;
use super::{Converter, ConverterInfo};

pub struct Ansize;

static INFO: ConverterInfo = ConverterInfo {
    id: "ansize",
    name: "ansize",
    origin: "github.com/jhchen/ansize",
    copyright: "Copyright (c) 2013 Jason Chen",
    license: "MIT",
    language: "Go",
    revision: "3bb41b2dc6e6e4b217b99c29d868fe8329d211ed",
    settings: "ansize <image> <output> 80",
    adaptations: "It picks each 0 or 1 with Go's time-seeded math/rand; here a fixed-seed SplitMix64 does, so the same image always gives the same file.",
    license_text: include_str!("licenses/ansize.txt"),
};

const PROPORTION: f32 = 0.46;
const CHARACTERS: &[u8] = b"01";

/// `toAnsiSpace`: a 16-bit channel to 0–5.
fn space(v: u32) -> u32 {
    (6.0f32 * (v as f32 / 65536.0f32)) as u32
}

impl Converter for Ansize {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let (w, h) = (image.width() as f32, image.height() as f32);
        let height = ((columns as f32 * (h / w) * PROPORTION) as u32).max(1);
        let img = resize(columns, height, &GoImage::from_straight(image), Interp::Lanczos3);
        let mut rng = SplitMix64::new(0);
        let mut grid = Grid::new(img.w as usize, img.h as usize);
        for y in 0..img.h {
            for x in 0..img.w {
                let [r, g, b, _] = img.rgba(x, y);
                let code = 16 + space(r) * 36 + space(g) * 6 + space(b);
                if code != 16 {
                    let ch = CHARACTERS[rng.below(CHARACTERS.len() as u64) as usize];
                    grid.set(x as usize, y as usize, Cell::new(ch, Color::Xterm(code as u8), Color::Default));
                }
            }
        }
        grid
    }
}
