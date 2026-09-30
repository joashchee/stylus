//! KOAN.ansi by Alexander Mitchell (github.com/koan-shdw/koan-ansi, MIT),
//! ported from `koan_ansi/convert.py`, `charset.py` and `palette.py`.
//!
//! Each cell is split into 2×2 quadrants. Every (glyph, foreground,
//! background) combination of the classic CP437 blocks and shades over
//! the 16 VGA colors is tried, and the one with the least channel-weighted
//! squared error wins, plus a penalty on shade glyphs that mix two
//! hue-distant colors (which read as speckle, not as their average).

use image::RgbaImage;

use super::grid::{Cell, Color, Grid};
use super::util::{drop_alpha, resample, Kernel};
use super::{Converter, ConverterInfo};

pub struct KoanAnsi;

static INFO: ConverterInfo = ConverterInfo {
    id: "koan-ansi",
    name: "KOAN.ansi",
    origin: "github.com/koan-shdw/koan-ansi",
    copyright: "Copyright (c) 2026 Alexander Mitchell (KOAN)",
    license: "MIT",
    language: "Python",
    revision: "eabd51f62ea2da596e0c8f0ef3c17788be34c840",
    settings: "classic charset (CP437 blocks and shades), vga16 palette with iCE colors, no dithering",
    adaptations: "None beyond the shared width.",
    license_text: include_str!("licenses/koan-ansi.txt"),
};

/// Its DOS-order VGA palette (palette.py).
const PALETTE: [[f64; 3]; 16] = [
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 170.0],
    [0.0, 170.0, 0.0],
    [0.0, 170.0, 170.0],
    [170.0, 0.0, 0.0],
    [170.0, 0.0, 170.0],
    [170.0, 85.0, 0.0],
    [170.0, 170.0, 170.0],
    [85.0, 85.0, 85.0],
    [85.0, 85.0, 255.0],
    [85.0, 255.0, 85.0],
    [85.0, 255.0, 255.0],
    [255.0, 85.0, 85.0],
    [255.0, 85.0, 255.0],
    [255.0, 255.0, 85.0],
    [255.0, 255.0, 255.0],
];

/// Per-channel weights for the squared RGB error (R, G, B).
const CHANNEL_W: [f64; 3] = [2.0, 4.0, 3.0];
const NOISE_K: f64 = 0.12;

/// CLASSIC in charset.py: CP437 byte, foreground coverage per quadrant
/// (top left, top right, bottom left, bottom right), shade mix if a shade.
const CLASSIC: [(u8, [f64; 4], Option<f64>); 9] = [
    (b' ', [0.0, 0.0, 0.0, 0.0], None),
    (0xB0, [0.25; 4], Some(0.25)),
    (0xB1, [0.5; 4], Some(0.5)),
    (0xB2, [0.75; 4], Some(0.75)),
    (0xDB, [1.0; 4], None),
    (0xDF, [1.0, 1.0, 0.0, 0.0], None),
    (0xDC, [0.0, 0.0, 1.0, 1.0], None),
    (0xDD, [1.0, 0.0, 1.0, 0.0], None),
    (0xDE, [0.0, 1.0, 0.0, 1.0], None),
];

struct Candidate {
    /// Reconstruction × channel weights, quadrant-major.
    recon_w: [f64; 12],
    /// Σ w·r² plus the noise penalty.
    konst: f64,
    glyph: u8,
    fg: u8,
    bg: u8,
}

/// Squared distance between two palette colors in an opponent space with
/// luma removed, for the noise penalty.
fn op_dist2(a: usize, b: usize) -> f64 {
    let op = |c: [f64; 3]| [c[0] - c[1], c[1] - c[2], c[2] - c[0]];
    let (x, y) = (op(PALETTE[a]), op(PALETTE[b]));
    (0..3).map(|i| (x[i] - y[i]).powi(2)).sum()
}

fn candidates() -> Vec<Candidate> {
    let mut out = Vec::new();
    for &(glyph, cov, shade) in &CLASSIC {
        let mix = shade.map_or(0.0, |s| s * (1.0 - s));
        let fgs = if cov == [0.0; 4] { 0..1 } else { 0..16 };
        for fg in fgs {
            let bgs = if cov == [1.0; 4] { 0..1 } else { 0..16 };
            for bg in bgs {
                let mut recon_w = [0.0; 12];
                let mut konst = NOISE_K * mix * 4.0 * op_dist2(fg, bg);
                for k in 0..4 {
                    for ch in 0..3 {
                        let r = PALETTE[fg][ch] * cov[k] + PALETTE[bg][ch] * (1.0 - cov[k]);
                        recon_w[k * 3 + ch] = r * CHANNEL_W[ch];
                        konst += r * r * CHANNEL_W[ch];
                    }
                }
                out.push(Candidate {
                    recon_w,
                    konst,
                    glyph,
                    fg: fg as u8,
                    bg: bg as u8,
                });
            }
        }
    }
    out
}

impl Converter for KoanAnsi {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        let (w, h) = image.dimensions();
        // auto_rows: cells are 1:2.
        let rows = ((columns as f64 * (h as f64 / w as f64) / 2.0).round() as u32).max(1);
        let (w2, h2) = (columns * 2, rows * 2);
        // quad_grid: BOX when shrinking, BICUBIC when a small source is enlarged.
        let kernel = if w >= w2 && h >= h2 { Kernel::Box } else { Kernel::Bicubic };
        let small = resample(&drop_alpha(image), w2, h2, kernel);
        let cands = candidates();
        let mut grid = Grid::new(columns as usize, rows as usize);
        for cy in 0..rows {
            for cx in 0..columns {
                let mut q = [0.0; 12];
                for k in 0..4 {
                    let p = small.get_pixel(cx * 2 + (k & 1), cy * 2 + (k >> 1));
                    for ch in 0..3 {
                        q[k as usize * 3 + ch] = p[ch] as f64;
                    }
                }
                let best = cands
                    .iter()
                    .min_by(|a, b| {
                        let score = |c: &Candidate| c.konst - 2.0 * (0..12).map(|j| q[j] * c.recon_w[j]).sum::<f64>();
                        score(a).total_cmp(&score(b))
                    })
                    .expect("candidates");
                grid.set(cx as usize, cy as usize, Cell::new(best.glyph, Color::dos(best.fg), Color::dos(best.bg)));
            }
        }
        grid
    }
}
