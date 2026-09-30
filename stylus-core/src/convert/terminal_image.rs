//! terminal-image by Sindre Sorhus (github.com/sindresorhus/terminal-image,
//! MIT), ported from `index.js` (`render`, `calculateWidthHeight`), with
//! Jimp's default resize (`@jimp/plugin-resize`'s `modules/resize.js`, a
//! port of Grant Galitz's JS-Image-Resizer, MIT), which it resizes with.
//!
//! Half blocks in 24-bit color: ▄ with the top pixel as background and the
//! bottom as foreground.

use image::RgbaImage;

use super::grid::{cp437, Cell, Color, Grid};
use super::{Converter, ConverterInfo};

pub struct TerminalImage;

static INFO: ConverterInfo = ConverterInfo {
    id: "terminal-image",
    name: "terminal-image",
    origin: "github.com/sindresorhus/terminal-image",
    copyright: "Copyright (c) Sindre Sorhus <sindresorhus@gmail.com> (https://sindresorhus.com)",
    license: "MIT",
    language: "JavaScript",
    revision: "f1f2f9aed0a6b128bcc5c19eb9ac24afe74b9e19",
    settings: "terminalImage.buffer(image, {width: 80, height: 10000, preferNativeRender: false}) in an 80-column true-color terminal",
    adaptations: "Jimp decodes JPEGs with its own decoder; here the shared one does, so JPEG colors can differ slightly.",
    license_text: include_str!("licenses/terminal-image.txt"),
};

/// JS `Math.round`: halves go up.
fn js_round(v: f64) -> f64 {
    (v + 0.5).floor()
}

/// Storing a number in a `Uint8Array`: truncated, modulo 256.
fn to_u8(v: f64) -> u8 {
    if v.is_finite() {
        (v.trunc() as i64).rem_euclid(256) as u8
    } else {
        0
    }
}

/// Jimp's `Resize` with `blendAlpha` and `interpolationPass` on: the width
/// pass into a Float32 buffer, then the height pass into bytes.
fn jimp_resize(src: &RgbaImage, tw: usize, th: usize) -> RgbaImage {
    let (w0, h0) = (src.width() as usize, src.height() as usize);
    let data: Vec<f32> = src.as_raw().iter().map(|&v| v as f32).collect();
    let width_pass: Vec<f32> = if w0 == tw {
        data
    } else {
        let ratio = w0 as f64 / tw as f64;
        let mut out = vec![0f32; tw * 4 * h0];
        if ratio < 1.0 {
            // resizeWidthInterpolatedRGBA
            let mut weight = 0.0;
            let mut t = 0;
            while weight < 1.0 / 3.0 {
                for y in 0..h0 {
                    for c in 0..4 {
                        out[y * tw * 4 + t + c] = src.as_raw()[y * w0 * 4 + c] as f32;
                    }
                }
                t += 4;
                weight += ratio;
            }
            weight -= 1.0 / 3.0;
            while weight < (w0 - 1) as f64 {
                let second = weight % 1.0;
                let first = 1.0 - second;
                let p = weight.floor() as usize * 4;
                for y in 0..h0 {
                    for c in 0..4 {
                        let a = src.as_raw()[y * w0 * 4 + p + c] as f64;
                        let b = src.as_raw()[y * w0 * 4 + p + 4 + c] as f64;
                        out[y * tw * 4 + t + c] = (a * first + b * second) as f32;
                    }
                }
                t += 4;
                weight += ratio;
            }
            while t < tw * 4 {
                for y in 0..h0 {
                    for c in 0..4 {
                        out[y * tw * 4 + t + c] = src.as_raw()[y * w0 * 4 + (w0 - 1) * 4 + c] as f32;
                    }
                }
                t += 4;
            }
        } else {
            // resizeWidthRGBA
            let divisor = 1.0 / ratio;
            let mut acc = vec![0f32; h0 * 4];
            let mut trust = vec![0f64; h0];
            let (mut actual, mut current) = (0usize, 0f64);
            let mut out_offset = 0;
            loop {
                acc.iter_mut().for_each(|v| *v = 0.0);
                trust.iter_mut().for_each(|v| *v = 0.0);
                let mut weight = ratio;
                loop {
                    let amount_to_next = 1.0 + actual as f64 - current;
                    let multiplier = weight.min(amount_to_next);
                    for y in 0..h0 {
                        let p = &src.as_raw()[y * w0 * 4 + actual..y * w0 * 4 + actual + 4];
                        let a = p[3];
                        for c in 0..3 {
                            acc[y * 4 + c] = (acc[y * 4 + c] as f64 + (if a != 0 { p[c] as f64 } else { 0.0 }) * multiplier) as f32;
                        }
                        acc[y * 4 + 3] = (acc[y * 4 + 3] as f64 + a as f64 * multiplier) as f32;
                        trust[y] += if a != 0 { multiplier } else { 0.0 };
                    }
                    if weight >= amount_to_next {
                        actual += 4;
                        current = actual as f64;
                        weight -= amount_to_next;
                    } else {
                        current += weight;
                        break;
                    }
                    if !(weight > 0.0 && actual < w0 * 4) {
                        break;
                    }
                }
                for y in 0..h0 {
                    let m = if trust[y] != 0.0 { 1.0 / trust[y] } else { 0.0 };
                    for c in 0..3 {
                        out[y * tw * 4 + out_offset + c] = (acc[y * 4 + c] as f64 * m) as f32;
                    }
                    out[y * tw * 4 + out_offset + 3] = (acc[y * 4 + 3] as f64 * divisor) as f32;
                }
                out_offset += 4;
                if out_offset >= tw * 4 {
                    break;
                }
            }
        }
        out
    };

    let row = tw * 4;
    let final_size = row * th;
    let bytes: Vec<u8> = if h0 == th {
        // Buffer.from(Float32Array): each value truncated to a byte.
        width_pass.iter().map(|&v| to_u8(v as f64)).collect()
    } else {
        let ratio = h0 as f64 / th as f64;
        let mut out = vec![0u8; final_size];
        if ratio < 1.0 {
            // resizeHeightInterpolated
            let mut weight = 0.0;
            let mut f = 0;
            while weight < 1.0 / 3.0 {
                for p in 0..row {
                    out[f] = to_u8(js_round(width_pass[p] as f64));
                    f += 1;
                }
                weight += ratio;
            }
            weight -= 1.0 / 3.0;
            let stop = h0 - 1;
            while weight < stop as f64 {
                let second = weight % 1.0;
                let first = 1.0 - second;
                let a = weight.floor() as usize * row;
                for p in 0..row {
                    out[f] = to_u8(js_round(width_pass[a + p] as f64 * first + width_pass[a + row + p] as f64 * second));
                    f += 1;
                }
                weight += ratio;
            }
            while f < final_size {
                for p in 0..row {
                    out[f] = to_u8(js_round(width_pass[stop * row + p] as f64));
                    f += 1;
                }
            }
        } else {
            // resizeHeightRGBA
            let divisor = 1.0 / ratio;
            let mut acc = vec![0f32; row];
            let mut trust = vec![0f64; tw];
            let (mut actual, mut current) = (0usize, 0f64);
            let mut out_offset = 0;
            loop {
                acc.iter_mut().for_each(|v| *v = 0.0);
                trust.iter_mut().for_each(|v| *v = 0.0);
                let mut weight = ratio;
                loop {
                    let amount_to_next = 1.0 + actual as f64 - current;
                    let multiplier = weight.min(amount_to_next);
                    for x in 0..tw {
                        let p = &width_pass[actual + x * 4..actual + x * 4 + 4];
                        let a = p[3];
                        for c in 0..3 {
                            acc[x * 4 + c] = (acc[x * 4 + c] as f64 + (if a != 0.0 { p[c] as f64 } else { 0.0 }) * multiplier) as f32;
                        }
                        acc[x * 4 + 3] = (acc[x * 4 + 3] as f64 + a as f64 * multiplier) as f32;
                        trust[x] += if a != 0.0 { multiplier } else { 0.0 };
                    }
                    let caret = actual + row;
                    if weight >= amount_to_next {
                        actual = caret;
                        current = actual as f64;
                        weight -= amount_to_next;
                    } else {
                        current += weight;
                        break;
                    }
                    if !(weight > 0.0 && actual < row * h0) {
                        break;
                    }
                }
                for x in 0..tw {
                    let m = if trust[x] != 0.0 { 1.0 / trust[x] } else { 0.0 };
                    for c in 0..3 {
                        out[out_offset] = to_u8(js_round(acc[x * 4 + c] as f64 * m));
                        out_offset += 1;
                    }
                    out[out_offset] = to_u8(js_round(acc[x * 4 + 3] as f64 * divisor));
                    out_offset += 1;
                }
                if out_offset >= final_size {
                    break;
                }
            }
        }
        out
    };
    RgbaImage::from_raw(tw as u32, th as u32, bytes).expect("resized size")
}

impl Converter for TerminalImage {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        // calculateWidthHeight with a width and a very tall height: the
        // width binds, and both are rounded.
        let (iw, ih) = (image.width() as f64, image.height() as f64);
        let factor = columns as f64 / iw;
        let width = js_round(factor * iw).max(1.0) as usize;
        let height = js_round(factor * ih).max(1.0) as usize;
        let img = jimp_resize(image, width, height);
        let rows = (height / 2).max(1);
        let mut grid = Grid::new(width, rows);
        for row in 0..rows {
            let y = row as u32 * 2;
            for x in 0..width as u32 {
                let top = img.get_pixel(x, y);
                // A 1-pixel-tall image has no second row: its own pixel stands in.
                let bottom = img.get_pixel(x, (y + 1).min(img.height() - 1));
                let fg = Color::Rgb(bottom[0], bottom[1], bottom[2]);
                let bg = if top[3] == 0 { Color::Default } else { Color::Rgb(top[0], top[1], top[2]) };
                grid.set(x as usize, row, Cell::new(cp437::LOWER_HALF, fg, bg));
            }
        }
        grid
    }
}
