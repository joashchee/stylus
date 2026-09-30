//! ansipx by Andrew Albers (github.com/Zebbeni/ansipx, MIT), the library
//! behind ANSIzalizer, ported from `unicode.go`, `renderer.go` and
//! `ansi.go`, with the parts of go-colorful (MIT) it calls.
//!
//! Its default half-block mode with a 16-color palette: each cell covers
//! 2×2 pixels, ▀ in the average of the top two over the average of the
//! bottom two, each average taken to the nearest palette color.

use image::RgbaImage;

use super::grid::{cp437, Cell, Color, Grid};
use super::nfnt::{resize, GoImage, Interp};
use super::util::VGA_SGR;
use super::{Converter, ConverterInfo};

pub struct Ansipx;

static INFO: ConverterInfo = ConverterInfo {
    id: "ansipx",
    name: "ansipx (ANSIzalizer)",
    origin: "github.com/Zebbeni/ansipx",
    copyright: "Copyright (c) 2024 Andrew Albers",
    license: "MIT",
    language: "Go",
    revision: "81f7fc599335e98adf4c2837313d70f42e9ce93d",
    settings: "DefaultOptions() (Unicode half blocks, Fit, CharRatio 0.46, nearest-neighbor sampling) with Width 80 and no height limit, TrueColor false, Palette the 16 VGA colors, OutputAlpha false",
    adaptations: "Dithering is off: ansipx dithers with makeworld-the-better-one/dither, which is MPL-2.0. OutputAlpha is off because its transparency edges use quarter blocks CP437 doesn't have; transparent pixels come out black instead.",
    license_text: include_str!("licenses/ansipx.txt"),
};

/// go-colorful's `Color`: RGB in 0–1.
type Rgb = [f64; 3];

/// `colorful.MakeColor` of a Go color's premultiplied 16-bit channels:
/// un-premultiplied again; black for a transparent one.
fn make_color([r, g, b, a]: [u32; 4]) -> Rgb {
    if a == 0 {
        return [0.0; 3];
    }
    [r, g, b].map(|v| (v * 0xffff / a) as f64 / 65535.0)
}

/// `color.Palette.Index` of a colorful color (opaque, channels
/// `uint32(v*65535 + 0.5)`), over the VGA colors.
fn palette_index(c: Rgb) -> usize {
    let c = c.map(|v| (v * 65535.0 + 0.5) as u32);
    let sq_diff = |x: u32, y: u32| {
        let d = x.wrapping_sub(y);
        d.wrapping_mul(d) >> 2
    };
    let mut best = 0;
    let mut best_sum = u32::MAX;
    for (i, p) in VGA_SGR.iter().enumerate() {
        let sum = (0..3).fold(0u32, |s, ch| s.wrapping_add(sq_diff(c[ch], p[ch] as u32 * 0x101)));
        if sum < best_sum {
            if sum == 0 {
                return i;
            }
            best = i;
            best_sum = sum;
        }
    }
    best
}

/// `avgCol` in palette mode: the average, to the nearest palette color.
fn avg_col(colors: &[Rgb]) -> usize {
    let n = colors.len() as f64;
    let sum = colors.iter().fold([0.0; 3], |s, c| [s[0] + c[0], s[1] + c[1], s[2] + c[2]]);
    palette_index(sum.map(|v| v / n))
}

/// Go's `color.NRGBAModel`, as `clearTransparentRGB`'s `dst.Set` stores a
/// resized pixel, read back with `RGBA()`.
fn through_nrgba([r, g, b, a]: [u32; 4]) -> [u32; 4] {
    let n = if a == 0xffff {
        [r >> 8, g >> 8, b >> 8, 0xff]
    } else if a == 0 {
        [0; 4]
    } else {
        [(r * 0xffff / a) >> 8, (g * 0xffff / a) >> 8, (b * 0xffff / a) >> 8, a >> 8]
    };
    let a = n[3] * 0x101;
    [n[0] * 0x101 * a / 0xffff, n[1] * 0x101 * a / 0xffff, n[2] * 0x101 * a / 0xffff, a]
}

impl Converter for Ansipx {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        // `processUnicode`'s Fit, in float32.
        let (img_w, img_h) = (image.width() as f32, image.height() as f32);
        let width = columns as usize;
        let height = ((columns as f32 * (img_h / img_w) * 0.46f32) as usize).max(1);
        let img = resize(width as u32 * 2, height as u32 * 2, &GoImage::from_straight(image), Interp::NearestNeighbor);
        let at = |x: usize, y: usize| make_color(through_nrgba(img.rgba(x as u32, y as u32)));
        let mut grid = Grid::new(width, height);
        for row in 0..height {
            let y = row * 2;
            for col in 0..width {
                let x = col * 2;
                // `calcTop`. (Its black-pixel tweak sets R to G, both 0.)
                let fg = avg_col(&[at(x, y), at(x + 1, y)]);
                let bg = avg_col(&[at(x, y + 1), at(x + 1, y + 1)]);
                grid.set(col, row, Cell::new(cp437::UPPER_HALF, Color::Ansi(fg as u8), Color::Ansi(bg as u8)));
            }
        }
        grid
    }
}
