//! libcaca's img2txt by Sam Hocevar (github.com/cacalabs/libcaca, WTFPL),
//! ported from `caca/dither.c` (`caca_dither_bitmap` with its defaults),
//! `src/img2txt.c` and the "ansi" exporter in `caca/codec/text.c`.
//!
//! Each cell averages the pixels under it ("prefilter"), then picks the
//! nearest of 16 colors as background and the next nearest as foreground,
//! and the glyph from its ASCII ramp whose fg/bg mix best matches, with
//! Floyd–Steinberg error diffusion along and down the rows ("fstein").

use image::RgbaImage;

use super::grid::{Cell, Color, Grid};
use super::{Converter, ConverterInfo};

pub struct Libcaca;

static INFO: ConverterInfo = ConverterInfo {
    id: "libcaca",
    name: "libcaca img2txt",
    origin: "github.com/cacalabs/libcaca",
    copyright: "Copyright (C) 2002-2018 Sam Hocevar",
    license: "WTFPL",
    language: "C",
    revision: "de48e1a0888caabdc6276afdea46e9096d0adb05",
    settings: "img2txt -W 80 -f ansi: fstein dithering, full16 colors, ascii glyphs, prefilter antialiasing, 6x10 font aspect",
    adaptations: "Images are read as 32-bit ARGB, as img2txt reads them through Imlib2.",
    license_text: include_str!("licenses/libcaca.txt"),
};

/// `rgb_palette`, 12-bit channels, in libcaca's (DOS) color order.
const RGB_PALETTE: [[i32; 3]; 16] = [
    [0x0, 0x0, 0x0],
    [0x0, 0x0, 0x7ff],
    [0x0, 0x7ff, 0x0],
    [0x0, 0x7ff, 0x7ff],
    [0x7ff, 0x0, 0x0],
    [0x7ff, 0x0, 0x7ff],
    [0x7ff, 0x7ff, 0x0],
    [0xaaa, 0xaaa, 0xaaa],
    [0x555, 0x555, 0x555],
    [0x000, 0x000, 0xfff],
    [0x000, 0xfff, 0x000],
    [0x000, 0xfff, 0xfff],
    [0xfff, 0x000, 0x000],
    [0xfff, 0x000, 0xfff],
    [0xfff, 0xfff, 0x000],
    [0xfff, 0xfff, 0xfff],
];

/// `ascii_glyphs` (the last, '?', is never picked: the search stops before it).
const GLYPHS: [u8; 11] = [b' ', b'.', b':', b';', b't', b'%', b'S', b'X', b'@', b'8', b'?'];

const FONT_WIDTH: u64 = 6;
const FONT_HEIGHT: u64 = 10;

fn sq(x: i32) -> i32 {
    x * x
}

impl Converter for Libcaca {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let (w, h) = (image.width() as u64, image.height() as u64);
        let cols = columns as u64;
        // img2txt: lines = cols * h * font_width / w / font_height (unsigned).
        let lines = (cols * h * FONT_WIDTH / w / FONT_HEIGHT).max(1);
        let mut grid = Grid::new(cols as usize, lines as usize);
        // Cells a transparent area skips keep the blank canvas: exported as
        // light gray on black.
        grid.cells.fill(Cell::new(b' ', Color::dos(7), Color::dos(0)));

        let dchmax = GLYPHS.len() as i32;
        let k = 2 * dchmax - 1;
        // fs_r/g/b with one guard cell on each side.
        let fs_len = cols as usize + 2;
        let mut fs = [vec![0i32; fs_len + 1], vec![0i32; fs_len + 1], vec![0i32; fs_len + 1]];
        for y in 0..lines {
            let mut remain = [0i32; 3];
            for x in 0..cols {
                // Prefilter: average the pixels this cell covers (8-bit
                // channels shifted to 12 bits by the masks).
                let from_x = x * w / cols;
                let from_y = y * h / lines;
                let mut to_x = (x + 1) * w / cols;
                let mut to_y = (y + 1) * h / lines;
                if to_x == from_x {
                    to_x += 1;
                }
                if to_y == from_y {
                    to_y += 1;
                }
                let mut rgba = [0u32; 4];
                let mut dots = 0u32;
                for my_x in from_x..to_x {
                    for my_y in from_y..to_y {
                        dots += 1;
                        // Past the edge (a 1-pixel image stretched) reads the
                        // last pixel.
                        let p = image.get_pixel(my_x.min(w - 1) as u32, my_y.min(h - 1) as u32);
                        for c in 0..4 {
                            rgba[c] += (p[c] as u32) << 4;
                        }
                    }
                }
                let mut rgba: [i32; 4] = rgba.map(|v| (v / dots) as i32);
                let fx = x as usize + 1; // index into fs (guard cell first)
                if rgba[3] < 0x800 {
                    remain = [0; 3];
                    for f in fs.iter_mut() {
                        f[fx] = 0;
                    }
                    continue;
                }
                for c in 0..3 {
                    rgba[c] += remain[c];
                }
                let nearest = |skip: Option<usize>| {
                    let mut best = 0;
                    let mut distmin = i32::MAX;
                    for (i, p) in RGB_PALETTE.iter().enumerate() {
                        if Some(i) == skip {
                            continue;
                        }
                        let dist = sq(rgba[0] - p[0]) + sq(rgba[1] - p[1]) + sq(rgba[2] - p[2]);
                        if dist < distmin {
                            best = i;
                            distmin = dist;
                        }
                    }
                    best
                };
                let out_bg = nearest(None);
                let out_fg = nearest(Some(out_bg));
                let (bg, fg) = (RGB_PALETTE[out_bg], RGB_PALETTE[out_fg]);
                let mut ch = 0;
                let mut distmin = i32::MAX;
                for i in 0..dchmax - 1 {
                    let dist: i32 = (0..3).map(|c| (rgba[c] * k - (i * fg[c] + (k - i) * bg[c])).abs()).sum();
                    if dist < distmin {
                        ch = i;
                        distmin = dist;
                    }
                }
                let error: [i32; 3] = std::array::from_fn(|c| rgba[c] - (fg[c] * ch + bg[c] * (k - ch)) / k);
                for c in 0..3 {
                    remain[c] = fs[c][fx + 1] + 7 * error[c] / 16;
                    fs[c][fx - 1] += 3 * error[c] / 16;
                    fs[c][fx] = 5 * error[c] / 16;
                    fs[c][fx + 1] = error[c] / 16;
                }
                grid.set(x as usize, y as usize, Cell::new(GLYPHS[ch as usize], Color::dos(out_fg as u8), Color::dos(out_bg as u8)));
            }
        }
        grid
    }
}
