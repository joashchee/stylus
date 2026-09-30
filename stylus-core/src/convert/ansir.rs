//! ansir by Bill Dwyer (github.com/themadcreator/ansir, Apache-2.0),
//! ported from `src/renderers/shaded-block.coffee`, `src/png.coffee`,
//! `src/config.coffee` and `src/ansi-codes.coffee`, with chroma.js's HSL
//! mixing and CIE Lab (chroma-js 3.2.0, what its `>=1.x.x` dependency
//! installs today).
//!
//! Each pixel becomes one shade glyph (░ ▒ ▓ █) in one of the 8 basic
//! colors on the terminal's black: the four glyphs are taken as the color
//! mixed with black at 25/50/75/100% in HSL, and a pixel takes the entry
//! nearest in CIE Lab.

use image::RgbaImage;

use super::grid::{cp437, Cell, Color, Grid};
use super::{Converter, ConverterInfo};

pub struct Ansir;

static INFO: ConverterInfo = ConverterInfo {
    id: "ansir",
    name: "ansir",
    origin: "github.com/themadcreator/ansir",
    copyright: "Copyright Bill Dwyer",
    license: "Apache-2.0",
    language: "CoffeeScript",
    revision: "95009d9526d5b8be3ed1cfd0f51d0521716bc120",
    settings: "ansir --mode shaded --colors basic --background dark --width 80 --height (rows for 1:2 cells), alpha cutoff 0.95",
    adaptations: "Its render loop runs one column and one row past the image (inclusive ranges); those cells are left out, since an 81st column would wrap.",
    license_text: include_str!("licenses/ansir.txt"),
};

/// ANSI_COLORS_BASIC: chroma.js's named color and the SGR color it's
/// written as, in its order.
const BASIC: [([f64; 3], u8); 8] = [
    ([255.0, 0.0, 0.0], 1),     // red
    ([255.0, 255.0, 0.0], 3),   // yellow
    ([0.0, 128.0, 0.0], 2),     // green
    ([0.0, 255.0, 255.0], 6),   // cyan
    ([0.0, 0.0, 255.0], 4),     // blue
    ([255.0, 0.0, 255.0], 5),   // magenta
    ([0.0, 0.0, 0.0], 0),       // black
    ([255.0, 255.0, 255.0], 7), // white
];

const GLYPHS: [u8; 4] = [cp437::LIGHT_SHADE, cp437::MEDIUM_SHADE, cp437::DARK_SHADE, cp437::FULL_BLOCK];
const ALPHA_CUTOFF: f64 = 0.95;

/// chroma.js `rgb2hsl`: hue is NaN for grays.
fn rgb2hsl(c: [f64; 3]) -> [f64; 3] {
    let [r, g, b] = c.map(|v| v / 255.0);
    let (min, max) = (r.min(g).min(b), r.max(g).max(b));
    let l = (max + min) / 2.0;
    if max == min {
        return [f64::NAN, 0.0, l];
    }
    let s = if l < 0.5 { (max - min) / (max + min) } else { (max - min) / (2.0 - max - min) };
    let mut h = if r == max {
        (g - b) / (max - min)
    } else if g == max {
        2.0 + (b - r) / (max - min)
    } else {
        4.0 + (r - g) / (max - min)
    };
    h *= 60.0;
    if h < 0.0 {
        h += 360.0;
    }
    [h, s, l]
}

/// chroma.js 3's `rgb2lab`: sRGB to XYZ, Bradford-adapted to D65, to Lab.
fn lab(c: [f64; 3]) -> [f64; 3] {
    const M: [[f64; 3]; 3] = [
        [0.4124564390896922, 0.21267285140562253, 0.0193338955823293],
        [0.357576077643909, 0.715152155287818, 0.11919202588130297],
        [0.18043748326639894, 0.07217499330655958, 0.9503040785363679],
    ];
    const MA: [[f64; 3]; 3] = [[0.8951, -0.7502, 0.0389], [0.2664, 1.7135, -0.0685], [-0.1614, 0.0367, 1.0296]];
    const MAI: [[f64; 3]; 3] = [
        [0.9869929054667123, 0.43230526972339456, -0.008528664575177328],
        [-0.14705425642099013, 0.5183602715367776, 0.04004282165408487],
        [0.15996265166373125, 0.0492912282128556, 0.9684866957875502],
    ];
    const WHITE: [f64; 3] = [0.95047, 1.0, 1.08883];
    const S: [f64; 3] = [0.9414285350000001, 1.040417467, 1.089532651];
    let gamma = |v: f64| {
        let v = v / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    // Row vector × matrix, as chroma.js multiplies (m00·x + m10·y + m20·z).
    let mul = |m: &[[f64; 3]; 3], v: [f64; 3]| -> [f64; 3] { std::array::from_fn(|j| v[0] * m[0][j] + v[1] * m[1][j] + v[2] * m[2][j]) };
    let xyz = mul(&M, [gamma(c[0]), gamma(c[1]), gamma(c[2])]);
    let d = mul(&MA, WHITE);
    let mut cone = mul(&MA, xyz);
    for i in 0..3 {
        cone[i] *= d[i] / S[i];
    }
    let xyz = mul(&MAI, cone);
    let (ke, kk) = (216.0 / 24389.0, 24389.0 / 27.0);
    let f = |t: f64| if t > ke { t.powf(1.0 / 3.0) } else { (kk * t + 16.0) / 116.0 };
    let (fx, fy, fz) = (f(xyz[0] / WHITE[0]), f(xyz[1] / WHITE[1]), f(xyz[2] / WHITE[2]));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

/// chroma.js 3's `hsl2rgb` (not rounded).
fn hsl2rgb([h, s, l]: [f64; 3]) -> [f64; 3] {
    if s == 0.0 {
        return [l * 255.0; 3];
    }
    let t2 = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let t1 = 2.0 * l - t2;
    let h = h / 360.0;
    let mut t3 = [h + 1.0 / 3.0, h, h - 1.0 / 3.0];
    let mut c = [0.0; 3];
    for i in 0..3 {
        if t3[i] < 0.0 {
            t3[i] += 1.0;
        }
        if t3[i] > 1.0 {
            t3[i] -= 1.0;
        }
        c[i] = if 6.0 * t3[i] < 1.0 {
            t1 + (t2 - t1) * 6.0 * t3[i]
        } else if 2.0 * t3[i] < 1.0 {
            t2
        } else if 3.0 * t3[i] < 2.0 {
            t1 + (t2 - t1) * ((2.0 / 3.0) - t3[i]) * 6.0
        } else {
            t1
        };
    }
    c.map(|v| v * 255.0)
}

/// chroma.js `mix(a, b, f, 'hsl')` (its `_hsx` interpolation).
fn mix_hsl(a: [f64; 3], b: [f64; 3], f: f64) -> [f64; 3] {
    let [h0, s0, l0] = rgb2hsl(a);
    let [h1, s1, l1] = rgb2hsl(b);
    let mut sat = None;
    let hue = if !h0.is_nan() && !h1.is_nan() {
        let dh = if h1 > h0 && h1 - h0 > 180.0 {
            h1 - (h0 + 360.0)
        } else if h1 < h0 && h0 - h1 > 180.0 {
            h1 + 360.0 - h0
        } else {
            h1 - h0
        };
        h0 + f * dh
    } else if !h0.is_nan() {
        if l1 == 1.0 || l1 == 0.0 {
            sat = Some(s0);
        }
        h0
    } else if !h1.is_nan() {
        if l0 == 1.0 || l0 == 0.0 {
            sat = Some(s1);
        }
        h1
    } else {
        f64::NAN
    };
    let sat = sat.unwrap_or(s0 + f * (s1 - s0));
    let l = l0 + f * (l1 - l0);
    // NaN hue with saturation 0 is a gray.
    hsl2rgb([if hue.is_nan() { 0.0 } else { hue }, if hue.is_nan() { 0.0 } else { sat }, l])
}

/// The octree's entries, (Lab, glyph, SGR color), in insertion order; an
/// entry at exactly the same point as an earlier one is dropped, as its
/// octree drops collisions.
fn table() -> Vec<([f64; 3], u8, u8)> {
    let mut out: Vec<([f64; 3], u8, u8)> = Vec::new();
    for &(color, sgr) in &BASIC {
        for (i, &glyph) in GLYPHS.iter().enumerate() {
            let opacity = (i + 1) as f64 / 4.0;
            let p = lab(mix_hsl([0.0, 0.0, 0.0], color, opacity));
            if !out.iter().any(|e| e.0 == p) {
                out.push((p, glyph, sgr));
            }
        }
    }
    out
}

impl Converter for Ansir {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let (pw, ph) = image.dimensions();
        let rows = super::util::rows_for(pw, ph, columns, 0.5);
        let table = table();
        let mut grid = Grid::new(columns as usize, rows as usize);
        for y in 0..rows {
            for x in 0..columns {
                // createRescaledImage with --width and --height: point sampling.
                let sx = (x as u64 * pw as u64 / columns as u64) as u32;
                let sy = (y as u64 * ph as u64 / rows as u64) as u32;
                let p = image.get_pixel(sx, sy);
                if (p[3] as f64 / 255.0) < ALPHA_CUTOFF {
                    continue; // a space, no colors
                }
                let q = lab([p[0] as f64, p[1] as f64, p[2] as f64]);
                let mut best = &table[0];
                let mut best_d = f64::MAX;
                for e in &table {
                    let d = ((e.0[0] - q[0]).powi(2) + (e.0[1] - q[1]).powi(2) + (e.0[2] - q[2]).powi(2)).sqrt();
                    if d < best_d {
                        best_d = d;
                        best = e;
                    }
                }
                grid.set(x as usize, y as usize, Cell::new(best.1, Color::Ansi(best.2), Color::Default));
            }
        }
        grid
    }
}
