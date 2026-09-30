//! ascii-image-converter by Zoraiz Hassan
//! (github.com/TheZoraiz/ascii-image-converter, Apache-2.0), ported from
//! `image_manipulation/util.go` (`resizeImage`),
//! `image_manipulation/image_conversions.go` and
//! `image_manipulation/ascii_conversions.go`, with imaging's Lanczos resize.
//!
//! Colored ASCII: each pixel becomes a character from a 10-step ramp by its
//! gray level, in the pixel's own 24-bit color.

use image::RgbaImage;

use super::grid::{Cell, Color, Grid};
use super::util::imaging_lanczos;
use super::{Converter, ConverterInfo};

pub struct AsciiImageConverter;

static INFO: ConverterInfo = ConverterInfo {
    id: "ascii-image-converter",
    name: "ascii-image-converter",
    origin: "github.com/TheZoraiz/ascii-image-converter",
    copyright: "Copyright (c) 2021 Zoraiz Hassan",
    license: "Apache-2.0",
    language: "Go",
    revision: "d05a757c5e02ab23e97b6f6fca4e1fbeb10ab559",
    settings: "ascii-image-converter <image> -W 80 -C in a true-color terminal",
    adaptations: "Go decodes JPEGs itself; here the shared decoder does, so JPEG colors can differ slightly.",
    license_text: include_str!("licenses/ascii-image-converter.txt"),
};

const TABLE: &[u8] = b" .:-=+*#%@";

impl Converter for AsciiImageConverter {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let aspect = image.width() as f64 / image.height() as f64;
        let h = ((0.5 * ((columns as f64 / aspect) as i64) as f64) as u32).max(1);
        let img = imaging_lanczos(image, columns, h);
        let mut grid = Grid::new(columns as usize, h as usize);
        for (x, y, p) in img.enumerate_pixels() {
            // NRGBA.RGBA(): premultiplied 16-bit; /257 back to 8 bits.
            let a = p[3] as u32 * 0x101;
            let c16 = |v: u8| v as u32 * 0x101 * a / 0xffff;
            let (r, g, b) = (c16(p[0]), c16(p[1]), c16(p[2]));
            // color.GrayModel.
            let depth = ((19595 * r + 38470 * g + 7471 * b + (1 << 15)) >> 24) as f64;
            let index = if depth == 255.0 { TABLE.len() - 1 } else { (depth / 255.0 * TABLE.len() as f64) as usize };
            let color = Color::Rgb((r / 257) as u8, (g / 257) as u8, (b / 257) as u8);
            grid.set(x as usize, y as usize, Cell::new(TABLE[index], color, Color::Default));
        }
        grid
    }
}
