//! viuer by Atanas Yankov (github.com/atanunq/viuer, MIT), ported from
//! `src/printer/block.rs` and `src/printer/mod.rs` (the block printer).
//!
//! Half blocks in 24-bit color: two pixels per cell, the top as the
//! background and the bottom as the foreground of ▄, after a Catmull-Rom
//! resize. Transparency shows its gray checkerboard.

use image::imageops::FilterType;
use image::{Rgba, RgbaImage};

use super::grid::{cp437, Cell, Color, Grid};
use super::{Converter, ConverterInfo};

pub struct Viuer;

static INFO: ConverterInfo = ConverterInfo {
    id: "viuer",
    name: "viuer",
    origin: "github.com/atanunq/viuer",
    copyright: "Copyright (c) 2022 Atanas Yankov",
    license: "MIT",
    language: "Rust",
    revision: "ed329e5c01d0aa0f65eae25fdb34e8cb87e5eac5",
    settings: "block printer, Config { width: Some(80), truecolor: true, transparent: false }",
    adaptations: "viuer only shrinks, so an image narrower than 80 pixels is enlarged to 80 columns here. Its 256-color mode isn't used: it depends on ansi_colours, which is LGPL.",
    license_text: include_str!("licenses/viuer.txt"),
};

const CHECKER_LIGHT: [u8; 3] = [153, 153, 153];
const CHECKER_DARK: [u8; 3] = [102, 102, 102];

fn checkerboard(row: u32, col: u32) -> [u8; 3] {
    if row % 2 == col % 2 {
        CHECKER_DARK
    } else {
        CHECKER_LIGHT
    }
}

/// `color_from_pixel` / `transparency_color` in truecolor.
fn color(row: u32, col: u32, p: &Rgba<u8>) -> Color {
    let a = p[3];
    if a == 0 {
        let [r, g, b] = checkerboard(row, col);
        return Color::Rgb(r, g, b);
    }
    if a < 255 {
        let c = checkerboard(row, col);
        let over = |f: u8, b: u8| ((f as u16 * a as u16 + b as u16 * (255 - a as u16)) / 255) as u8;
        return Color::Rgb(over(p[0], c[0]), over(p[1], c[1]), over(p[2], c[2]));
    }
    Color::Rgb(p[0], p[1], p[2])
}

/// `fit_dimensions` for a width bound only, in cells. An image as wide as
/// the bound keeps its size; any other is scaled to the bound's width
/// (viuer leaves narrower ones alone).
fn fit(width: u32, height: u32, bound_width: u32) -> (u32, u32) {
    if width == bound_width {
        return (width, (height / 2 + height % 2).max(1));
    }
    let intermediate = (height as u64 * bound_width as u64 / width as u64) as u32;
    (bound_width, (intermediate / 2).max(1))
}

impl Converter for Viuer {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let (w, h) = fit(image.width(), image.height(), columns);
        let img = image::imageops::resize(image, w, (2 * h).saturating_sub(image.height() % 2).max(1), FilterType::CatmullRom);
        let height = img.height();
        let rows = height.div_ceil(2);
        let mut grid = Grid::new(w as usize, rows as usize);
        for row in 0..rows {
            let top = row * 2;
            for x in 0..w {
                let t = color(top, x, img.get_pixel(x, top));
                let cell = if top + 1 < height {
                    Cell::new(cp437::LOWER_HALF, color(top + 1, x, img.get_pixel(x, top + 1)), t)
                } else {
                    // The last, unpaired row: ▀ in the top color.
                    Cell::new(cp437::UPPER_HALF, t, Color::Default)
                };
                grid.set(x as usize, row as usize, cell);
            }
        }
        grid
    }
}
