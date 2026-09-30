//! img2ansi by Jakob Westhoff (github.com/jakobwesthoff/img2ansi, MIT),
//! ported from `src/main.rs` and `src/resizing.rs`.
//!
//! Half blocks in 24-bit color: ▀ with the upper pixel as the foreground
//! and the lower one as the background, after its own Lanczos resize
//! (a = 2), which reads a fixed 2×2 window at each point, however far the
//! image shrinks.

use image::RgbaImage;

use super::grid::{cp437, Cell, Color, Grid};
use super::util::drop_alpha;
use super::{Converter, ConverterInfo};

pub struct Img2AnsiJakobwesthoff;

static INFO: ConverterInfo = ConverterInfo {
    id: "img2ansi-jakobwesthoff",
    name: "img2ansi (Jakob Westhoff)",
    origin: "github.com/jakobwesthoff/img2ansi",
    copyright: "Copyright © 2024 Jakob Westhoff",
    license: "MIT",
    language: "Rust",
    revision: "f0ca51948e1b87b20265c68f67a47b5b6273b0dd",
    settings: "a terminal 80 columns wide and tall enough for the whole image; resize_lanczos with a = 2",
    adaptations: "The original reads one fixed file and sizes to the terminal; here it takes the image and 80 columns. It refuses images with an alpha channel, so alpha is ignored here, and it stops with an error on an odd number of pixel rows, so the last odd row is left out.",
    license_text: include_str!("licenses/img2ansi-jakobwesthoff.txt"),
};

fn sinc(x: f64) -> f64 {
    if x.abs() < f64::EPSILON {
        1.0
    } else {
        x.sin() / x
    }
}

/// `lanczos`: its window test (`-a < x || x < a`) is always true, so this
/// is sinc(x)·sinc(x/a) everywhere, with sinc unnormalized (no π).
fn lanczos(x: f64, a: f64) -> f64 {
    sinc(x) * sinc(x / a)
}

/// `resize_lanczos`, on RGB.
fn resize_lanczos(img: &RgbaImage, new_width: usize, new_height: usize, a: f64) -> Vec<[u8; 3]> {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let mut out = vec![[0u8; 3]; new_width * new_height];
    for ny in 0..new_height {
        let oy = ny as f64 * h as f64 / new_height as f64;
        for nx in 0..new_width {
            let ox = nx as f64 * w as f64 / new_width as f64;
            let (mut r, mut g, mut b, mut sum) = (0f64, 0f64, 0f64, 0f64);
            for iy in (oy.floor() - a + 1.0) as isize..(oy.floor() + a) as isize {
                for ix in (ox.floor() - a + 1.0) as isize..(ox.floor() + a) as isize {
                    if iy < 0 || iy >= h as isize || ix < 0 || ix >= w as isize {
                        continue;
                    }
                    let weight = lanczos(ox - ix as f64, a) * lanczos(oy - iy as f64, a);
                    sum += weight;
                    let p = img.get_pixel(ix as u32, iy as u32);
                    r += p[0] as f64 * weight;
                    g += p[1] as f64 * weight;
                    b += p[2] as f64 * weight;
                }
            }
            // An `as u8` of NaN (a zero weight sum) is 0, as in the original.
            let px = |v: f64| (v / sum).round().clamp(0.0, 255.0) as u8;
            out[ny * new_width + nx] = [px(r), px(g), px(b)];
        }
    }
    out
}

impl Converter for Img2AnsiJakobwesthoff {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let image = drop_alpha(image);
        let (w, h) = (image.width() as f64, image.height() as f64);
        // `min(width_ratio, height_ratio)` with no height bound.
        let ratio = columns as f64 / w;
        let tw = ((w * ratio).floor() as usize).max(1);
        let th = ((h * ratio).floor() as usize).max(2);
        let px = resize_lanczos(&image, tw, th, 2.0);
        let rows = th / 2;
        let mut grid = Grid::new(tw, rows);
        for row in 0..rows {
            for x in 0..tw {
                let [fr, fg, fb] = px[row * 2 * tw + x];
                let [br, bg, bb] = px[(row * 2 + 1) * tw + x];
                grid.set(x, row, Cell::new(cp437::UPPER_HALF, Color::Rgb(fr, fg, fb), Color::Rgb(br, bg, bb)));
            }
        }
        grid
    }
}
