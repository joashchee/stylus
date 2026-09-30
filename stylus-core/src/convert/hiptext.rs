//! hiptext by Justine Tunney (github.com/jart/hiptext, Apache-2.0),
//! ported from `src/hiptext.cc` (`PrintImageXterm256`), `src/artiste.cc`
//! (`ComputeDimensions`), `src/graphic.cc` (`BilinearScale`),
//! `src/pixel.cc` and `src/xterm256.cc`.
//!
//! One pixel per cell, as a no-break space on the nearest xterm color (the cube and
//! grays, by RGB distance), after a bilinear scale that samples the source
//! at each output pixel's corner (no averaging).

use image::RgbaImage;

use super::grid::{Cell, Color, Grid};
use super::util::xterm256;
use super::{Converter, ConverterInfo};

pub struct Hiptext;

static INFO: ConverterInfo = ConverterInfo {
    id: "hiptext",
    name: "hiptext",
    origin: "github.com/jart/hiptext",
    copyright: "By Justine Tunney",
    license: "Apache-2.0",
    language: "C++",
    revision: "35c91e0dc923eb12a7fb144a0dc6bc85001d9c79",
    settings: "hiptext --width 80 in an 80-column terminal tall enough for the whole image (xterm256, black background)",
    adaptations: "None beyond the shared width. Its terminal printer's quirk of sometimes leaving a cell on the default background, where the color matches one written before a reset, is kept.",
    license_text: include_str!("licenses/hiptext.txt"),
};

#[derive(Clone, Copy)]
struct Pixel([f64; 4]);

fn lerp(p1: f64, p2: f64, t: f64) -> f64 {
    p1 * (1.0 - t) + p2 * t
}

impl Converter for Hiptext {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let (w, h) = (image.width() as i64, image.height() as i64);
        // ComputeDimensions, not duo-pixel: rows are half the height.
        let ratio = w as f64 / h as f64;
        let mut width = columns as f64;
        let mut height = f64::MAX;
        height = height.min(width / ratio);
        width = width.min(height * ratio);
        let (nw, nh) = ((width as i64).max(1), ((height as i64) / 2).max(1));

        let src: Vec<Pixel> = image.pixels().map(|p| Pixel(p.0.map(|v| v as f64 / 255.0))).collect();
        let safe = |x: i64, y: i64| src[(y.clamp(0, h - 1) * w + x.clamp(0, w - 1)) as usize];
        let (rx, ry) = (w as f64 / nw as f64, h as f64 / nh as f64);
        let xterm: Vec<[f64; 3]> = (0..=255u8).map(|i| xterm256(i).map(|v| v as f64 / 255.0)).collect();
        let bg256 = 16; // rgb_to_xterm256(black)
        let mut grid = Grid::new(nw as usize, nh as usize);
        // TermPrinter's state: the background it last wrote (`cur_`), the
        // one asked for (`new_`, 0 for none), whether they differ
        // (`dirty_`), and the background the terminal ends up showing.
        let (mut cur, mut new, mut dirty, mut shown) = (None::<u8>, None::<u8>, false, Color::Default);
        for y in 0..nh {
            for x in 0..nw {
                // BilinearScale (the same size returns the image as is).
                let px = if (nw, nh) == (w, h) {
                    safe(x, y)
                } else {
                    let (sx, sy) = (x as f64 * rx, y as f64 * ry);
                    let (fx, fy) = (sx.floor() as i64, sy.floor() as i64);
                    let (tx, ty) = (sx - fx as f64, sy - fy as f64);
                    let (tl, tr, bl, br) = (safe(fx, fy), safe(fx + 1, fy), safe(fx, fy + 1), safe(fx + 1, fy + 1));
                    Pixel(std::array::from_fn(|c| lerp(lerp(tl.0[c], tr.0[c], tx), lerp(bl.0[c], br.0[c], tx), ty)))
                };
                // Opacify over black.
                let a = px.0[3];
                let rgb: [f64; 3] = if a == 1.0 {
                    [px.0[0], px.0[1], px.0[2]]
                } else if a == 0.0 {
                    [0.0; 3]
                } else {
                    std::array::from_fn(|c| px.0[c] * a + 0.0 * (1.0 - a))
                };
                // rgb_to_xterm(pix, 16, 256).
                let mut best = 0u8;
                let mut smallest = 1e9;
                for c in 16..256 {
                    let d = ((xterm[c][0] - rgb[0]).powi(2) + (xterm[c][1] - rgb[1]).powi(2) + (xterm[c][2] - rgb[2]).powi(2)).sqrt();
                    if d < smallest {
                        smallest = d;
                        best = c as u8;
                    }
                }
                // The terminal's own background stands in for black:
                // `SetBackground256(0)`, no background.
                let code = (best != bg256).then_some(best);
                if code != new {
                    new = code;
                    dirty = true;
                }
                // `Flush`, before each character. Going to no background
                // writes a reset but leaves `cur_` as it was, so a later
                // return to that color writes only `\e[m` (another reset)
                // and the cell keeps the default background.
                if dirty {
                    dirty = false;
                    shown = match new {
                        Some(c) if new != cur => {
                            cur = new;
                            Color::Xterm(c)
                        }
                        _ => Color::Default,
                    };
                }
                // `--space`'s default, U+00A0, is CP437 0xFF.
                grid.set(x as usize, y as usize, Cell::new(0xFF, Color::Default, shown));
            }
            // `Reset()` at the end of the row, only when styled.
            if new.is_some() {
                (cur, new, shown) = (None, None, Color::Default);
            }
        }
        grid
    }
}
