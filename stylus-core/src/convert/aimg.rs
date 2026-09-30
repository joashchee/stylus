//! aimg by Bjoern Oelke (github.com/stroborobo/aimg, ISC), ported from
//! `image.go`, with its ansirgb palette (github.com/stroborobo/ansirgb, ISC).
//!
//! Half blocks in 256 colors from point samples (no filtering): ▄ with the
//! top pixel as background and the bottom as foreground, each the nearest
//! of xterm's cube and grays (never the 16 system colors, which terminals
//! change); a cell whose two pixels match is a space.

use image::RgbaImage;

use super::grid::{cp437, Cell, Color, Grid};
use super::nfnt::GoImage;
use super::util::xterm256;
use super::{Converter, ConverterInfo};

pub struct Aimg;

static INFO: ConverterInfo = ConverterInfo {
    id: "aimg",
    name: "aimg",
    origin: "github.com/stroborobo/aimg",
    copyright: "Copyright (c) 2014, Bjoern Oelke",
    license: "ISC",
    language: "Go",
    revision: "8966abff05e4ae13293565ea0fc2b02295d11d86",
    settings: "aimg -w 81 -n (80 columns plus the reset column it keeps back)",
    adaptations: "Each line ends with a reset and a space against resize artifacts; that 81st column is left out, since it would wrap. An image narrower than 80 pixels is stretched to 80 columns (aimg keeps it at its own width).",
    license_text: include_str!("licenses/aimg.txt"),
};

/// ansirgb's palette: codes 16–255, then a transparent entry (code -1).
/// Go's `Palette.Index` over it, on premultiplied 16-bit channels.
fn convert(c: [u32; 4]) -> Option<u8> {
    let sq_diff = |x: u32, y: u32| {
        let d = x.wrapping_sub(y);
        d.wrapping_mul(d) >> 2
    };
    let entry = |i: usize| -> ([u32; 4], Option<u8>) {
        if i < 240 {
            let code = (i + 16) as u8;
            let [r, g, b] = xterm256(code).map(|v| v as u32 * 0x101);
            ([r, g, b, 0xffff], Some(code))
        } else {
            ([0; 4], None)
        }
    };
    let mut best = entry(0).1;
    let mut best_sum = u32::MAX;
    for i in 0..241 {
        let (v, code) = entry(i);
        let sum = (0..4).fold(0u32, |s, k| s.wrapping_add(sq_diff(c[k], v[k])));
        if sum < best_sum {
            if sum == 0 {
                return code;
            }
            best = code;
            best_sum = sum;
        }
    }
    best
}

impl Converter for Aimg {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let img = GoImage::from_straight(image);
        let ratio = img.w as f64 / columns as f64;
        let rows = ((img.h as f64 / ratio) as u32).max(1);
        // getColor: nearly transparent is the terminal's default.
        let color = |x: u32, y: u32| -> Option<u8> {
            if x >= img.w || y >= img.h {
                return None; // At() outside the image: transparent
            }
            let c = img.rgba(x, y);
            if c[3] < (1 << 14) - 1 {
                None
            } else {
                convert(c)
            }
        };
        let cells = rows.div_ceil(2);
        let mut grid = Grid::new(columns as usize, cells as usize);
        let as_color = |c: Option<u8>| c.map_or(Color::Default, Color::Xterm);
        for r in (0..rows).step_by(2) {
            for c in 0..columns {
                let x = (ratio * c as f64) as u32;
                let top = color(x, (ratio * r as f64) as u32);
                let bottom = color(x, (ratio * (r + 1) as f64) as u32);
                // Block.String: top is the background of ▄, unless the
                // bottom is transparent, which only a background can be.
                let (first, second, glyph) = if bottom.is_none() { (bottom, top, cp437::UPPER_HALF) } else { (top, bottom, cp437::LOWER_HALF) };
                let cell = if first == second { Cell::new(b' ', Color::Default, as_color(first)) } else { Cell::new(glyph, as_color(second), as_color(first)) };
                grid.set(c as usize, (r / 2) as usize, cell);
            }
        }
        grid
    }
}
