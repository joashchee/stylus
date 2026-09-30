//! image-to-ansi by Dom Hastings (github.com/dom111/image-to-ansi, MIT),
//! ported from `src/js/process.ts` and `src/js/rgbToTerm.ts`.
//!
//! Half blocks in 256 colors: ▄ with the top pixel as background and the
//! bottom as foreground, each the xterm color nearest by Manhattan
//! distance (the lower number on a tie); two equal pixels make a space.

use image::{Rgba, RgbaImage};

use super::grid::{cp437, Cell, Color, Grid};
use super::util::{resample, Kernel, CUBE_LEVELS, WINDOWS_SGR};
use super::{Converter, ConverterInfo};

pub struct ImageToAnsi;

static INFO: ConverterInfo = ConverterInfo {
    id: "image-to-ansi",
    name: "image-to-ansi",
    origin: "github.com/dom111/image-to-ansi",
    copyright: "Copyright (c) 2021 Dom Hastings",
    license: "MIT",
    language: "TypeScript",
    revision: "6a32f66b3904dc3835e48e6fab20945fb23bc7b7",
    settings: "Unicode on, 256 colours, max width 80",
    adaptations: "It scales the image on a browser canvas, which varies by browser; here a bilinear filter does. It only shrinks, so an image narrower than 80 pixels is enlarged to 80 columns here.",
    license_text: include_str!("licenses/image-to-ansi.txt"),
};

/// `buildColours`: (r, g, b, code) in its order.
fn colours() -> Vec<([i32; 3], u8)> {
    let mut out: Vec<([i32; 3], u8)> = WINDOWS_SGR.iter().enumerate().map(|(i, c)| (c.map(|v| v as i32), i as u8)).collect();
    for &r in &CUBE_LEVELS {
        for &g in &CUBE_LEVELS {
            for &b in &CUBE_LEVELS {
                // 16 + parseInt(floor(r/255*5) floor(g…) floor(b…), 6)
                let d = |v: u8| ((v as f64 / 255.0) * 5.0).floor() as u8;
                out.push(([r as i32, g as i32, b as i32], 16 + d(r) * 36 + d(g) * 6 + d(b)));
            }
        }
    }
    for s in (8..=238).step_by(10) {
        out.push(([s; 3], 232 + (s / 10) as u8));
    }
    out
}

/// `rgbToTerm`: least Manhattan distance, lower code first.
fn to_term(colours: &[([i32; 3], u8)], p: &Rgba<u8>) -> u8 {
    let c = [p[0] as i32, p[1] as i32, p[2] as i32];
    colours
        .iter()
        .min_by_key(|(v, code)| ((v[0] - c[0]).abs() + (v[1] - c[1]).abs() + (v[2] - c[2]).abs(), *code))
        .map_or(0, |(_, code)| *code)
}

/// Mostly transparent: under 0.05 opacity.
fn transparent(p: Option<&Rgba<u8>>) -> bool {
    p.is_none_or(|p| p[3] < 13)
}

impl Converter for ImageToAnsi {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let scale = image.width() as f64 / columns as f64;
        let height = ((image.height() as f64 / scale).floor() as u32).max(1);
        let img = resample(image, columns, height, Kernel::Triangle);
        let colours = colours();
        let color = |p: Option<&Rgba<u8>>| if transparent(p) { Color::Default } else { Color::Xterm(to_term(&colours, p.expect("opaque"))) };
        let rows = height.div_ceil(2);
        let mut grid = Grid::new(columns as usize, rows as usize);
        for row in 0..rows {
            for x in 0..columns {
                let top = Some(img.get_pixel(x, row * 2));
                let bottom = (row * 2 + 1 < height).then(|| img.get_pixel(x, row * 2 + 1));
                let same = matches!((top, bottom), (Some(t), Some(b)) if t[0] == b[0] && t[1] == b[1] && t[2] == b[2]);
                let cell = if (same && !transparent(top) && !transparent(bottom)) || (transparent(top) && transparent(bottom)) {
                    Cell::new(b' ', Color::Default, color(top))
                } else if transparent(bottom) {
                    Cell::new(cp437::UPPER_HALF, color(top), color(bottom))
                } else {
                    Cell::new(cp437::LOWER_HALF, color(bottom), color(top))
                };
                grid.set(x as usize, row as usize, cell);
            }
        }
        grid
    }
}
