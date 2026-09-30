//! ASCII Magic by Leandro Barone (github.com/LeandroBarone/python-ascii_magic,
//! MIT), ported from `ascii_magic/ascii_art.py` (`_img_to_art`,
//! `get_color_data`) and `constants.py`.
//!
//! Colored ASCII: each pixel picks a character from a ramp by its
//! brightness, and the foreground color of the 8 terminal colors that,
//! scaled by that brightness, is nearest the pixel in gamma-2.2 space.

use image::RgbaImage;

use super::grid::{Cell, Color, Grid};
use super::util::{resample, Kernel};
use super::{Converter, ConverterInfo};

pub struct AsciiMagic;

static INFO: ConverterInfo = ConverterInfo {
    id: "ascii-magic",
    name: "ASCII Magic",
    origin: "github.com/LeandroBarone/python-ascii_magic",
    copyright: "Copyright (c) 2020 Leandro Barone",
    license: "MIT",
    language: "Python",
    revision: "f78ff741fbb55f9c819dc1280fd88d6944e1cab7",
    settings: "AsciiArt.from_image(...).to_terminal(columns=80), width_ratio 2.2",
    adaptations: "Palette-based images (GIF, 8-bit PNG) are read as full color, so they're resized like any other instead of nearest-neighbor.",
    license_text: include_str!("licenses/ascii-magic.txt"),
};

const CHARS: &[u8] = b" .`-_':,;^=+/\"|)\\<>)iv%xclrs{*}I?!][1taeo7zjLunT#JCwfy325Fp6mqSghVd4EgXPGZbYkOA&8U$@KHDBWNMR0QQ";

/// PALETTE: the unit color and the SGR foreground it's written as.
const PALETTE: [([f64; 3], Color); 8] = [
    ([0.0, 0.0, 0.0], Color::Ansi(8)), // LIGHTBLACK (90)
    ([0.0, 0.0, 1.0], Color::Ansi(4)),
    ([0.0, 1.0, 0.0], Color::Ansi(2)),
    ([1.0, 0.0, 0.0], Color::Ansi(1)),
    ([1.0, 1.0, 1.0], Color::Ansi(7)),
    ([1.0, 0.0, 1.0], Color::Ansi(5)),
    ([0.0, 1.0, 1.0], Color::Ansi(6)),
    ([1.0, 1.0, 0.0], Color::Ansi(3)),
];

impl Converter for AsciiMagic {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let width_ratio = 2.2;
        let (w, h) = (image.width() as f64, image.height() as f64);
        let scalar = w * width_ratio / columns as f64;
        let img_w = ((w * width_ratio / scalar) as u32).max(1);
        let img_h = ((h / scalar) as u32).max(1);
        // Image.resize's default filter for RGB(A): bicubic.
        let img = resample(image, img_w, img_h, Kernel::Bicubic);
        let mut grid = Grid::new(img_w as usize, img_h as usize);
        for (x, y, p) in img.enumerate_pixels() {
            // convert("L"): Pillow's integer luma.
            let l = (p[0] as u32 * 19595 + p[1] as u32 * 38470 + p[2] as u32 * 7471 + 0x8000) >> 16;
            let brightness = l as f64 / 255.0;
            let rgb = [p[0], p[1], p[2]].map(|v| (v as f64 / 255.0).powf(2.2));
            let ch = CHARS[(brightness * (CHARS.len() - 1) as f64) as usize];
            let mut min = 2.0;
            let mut index = 0;
            for (i, (unit, _)) in PALETTE.iter().enumerate() {
                let d: f64 = (0..3).map(|c| (unit[c] * brightness - rgb[c]).powi(2)).sum();
                if d < min {
                    index = i;
                    min = d;
                }
            }
            grid.set(x as usize, y as usize, Cell::new(ch, PALETTE[index].1, Color::Default));
        }
        grid
    }
}
