//! termpix by hopey-dishwasher (github.com/hopey-dishwasher/termpix,
//! Apache-2.0), ported from `src/lib.rs` (`print_image`,
//! `find_colour_index`, `blend_alpha`) and `src/main.rs` (sizing).
//!
//! Half blocks in 256 colors: ▄ with the top pixel as background and the
//! bottom as foreground, each the nearest of its xterm table (colors 16 to
//! 254), after a nearest-neighbor resize; transparency is blended into a
//! dark gray.

use image::imageops::FilterType;
use image::{Rgba, RgbaImage};

use super::grid::{cp437, Cell, Color, Grid};
use super::util::windows_xterm256;
use super::{Converter, ConverterInfo};

pub struct Termpix;

static INFO: ConverterInfo = ConverterInfo {
    id: "termpix",
    name: "termpix",
    origin: "github.com/hopey-dishwasher/termpix",
    copyright: "Copyright 2016 hopey-dishwasher",
    license: "Apache-2.0",
    language: "Rust",
    revision: "c22d061fde753fe847b40b8ccdc3ad4515d2f47d",
    settings: "termpix <file> --width 80 (256 colors)",
    adaptations: "None beyond the shared width.",
    license_text: include_str!("licenses/termpix.txt"),
};

/// ANSI_COLOURS: xterm's, except two grays (241, 242) it lists as 0x60
/// and 0x66.
fn ansi_colour(i: u8) -> [i32; 3] {
    match i {
        241 => [0x60; 3],
        242 => [0x66; 3],
        _ => windows_xterm256(i).map(|v| v as i32),
    }
}

/// `find_colour_index`: 16 up to 254 (its range stops before 255).
fn find_colour_index(p: [u8; 3]) -> u8 {
    let mut best = 0;
    let mut best_distance = 255 * 255 * 3 + 1;
    for i in 16..255u8 {
        let c = ansi_colour(i);
        let d = (0..3).map(|k| (c[k] - p[k] as i32).pow(2)).sum::<i32>();
        if d < best_distance {
            best_distance = d;
            best = i;
        }
    }
    best
}

/// `blend_alpha`: over gray 38, in f32, truncated.
fn blend(p: &Rgba<u8>) -> [u8; 3] {
    let a = p[3] as f32 / 255.0;
    [0, 1, 2].map(|c| (a * p[c] as f32 + (1.0 - a) * 38.0) as u8)
}

impl Converter for Termpix {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        // scale_dimension: height in pixels, two per row.
        let height = ((image.height() as f32 * columns as f32 / image.width() as f32 + 0.5) as u32).max(1);
        let img = image::imageops::resize(image, columns, height, FilterType::Nearest);
        let rows = (height / 2).max(1);
        let mut grid = Grid::new(columns as usize, rows as usize);
        for row in 0..height / 2 {
            let y = row * 2;
            for x in 0..columns {
                let top = find_colour_index(blend(img.get_pixel(x, y)));
                let bottom = find_colour_index(blend(img.get_pixel(x, y + 1)));
                grid.set(x as usize, row as usize, Cell::new(cp437::LOWER_HALF, Color::Xterm(bottom), Color::Xterm(top)));
            }
        }
        grid
    }
}
