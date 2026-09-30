//! CLImage by Patrick Nappa (github.com/pnappa/CLImage, MIT), ported
//! from `climage/climage.py` (`convert` and `_toAnsi`).
//!
//! Half blocks: each cell is two pixels stacked, the top as the
//! background and the bottom as the foreground of ▄, each the nearest
//! color of the chosen system palette.

use image::RgbaImage;

use super::grid::{cp437, Cell, Color, Grid};
use super::util::{drop_alpha, py_floordiv, resample, Kernel, VGA_SGR};
use super::{Converter, ConverterInfo};

pub struct Climage;

static INFO: ConverterInfo = ConverterInfo {
    id: "climage",
    name: "CLImage",
    origin: "github.com/pnappa/CLImage",
    copyright: "Copyright (c) 2024 Patrick Nappa",
    license: "MIT",
    language: "Python",
    revision: "ad17f75ba1d59e2957227a15fdcb84d7448d4edb",
    settings: "convert(is_unicode=True, is_16color=True, palette=\"linuxconsole\", width=80)",
    adaptations: "None beyond the shared width.",
    license_text: include_str!("licenses/climage.txt"),
};

/// The `kdtree` package's tree (github.com/stefankoegl/kdtree, ISC),
/// which CLImage finds nearest colors with. Its search order decides
/// between equally near colors, which the VGA palette makes common, so
/// it's ported as is: built by a stable sort and median split per axis,
/// searched depth first, keeping the first node strictly nearer.
struct KdNode {
    point: [u8; 3],
    code: u8,
    axis: usize,
    left: Option<Box<KdNode>>,
    right: Option<Box<KdNode>>,
}

impl KdNode {
    fn create(mut points: Vec<([u8; 3], u8)>, axis: usize) -> Option<Box<KdNode>> {
        if points.is_empty() {
            return None;
        }
        points.sort_by_key(|p| p.0[axis]); // stable, as Python's sort
        let median = points.len() / 2;
        let right = points.split_off(median + 1);
        let (point, code) = points.pop().expect("median");
        Some(Box::new(KdNode {
            point,
            code,
            axis,
            left: KdNode::create(points, (axis + 1) % 3),
            right: KdNode::create(right, (axis + 1) % 3),
        }))
    }

    fn search(&self, p: [u8; 3], best: &mut Option<(i64, u8)>) {
        let d = |a: u8, b: u8| (a as i64 - b as i64).pow(2);
        let dist = d(self.point[0], p[0]) + d(self.point[1], p[1]) + d(self.point[2], p[2]);
        match best {
            Some((bd, _)) if dist >= *bd => {}
            _ => *best = Some((dist, self.code)),
        }
        let plane = d(self.point[self.axis], p[self.axis]);
        let (near, far) = if p[self.axis] < self.point[self.axis] { (&self.left, &self.right) } else { (&self.right, &self.left) };
        if let Some(n) = near {
            n.search(p, best);
        }
        if best.is_some_and(|(bd, _)| plane < bd) {
            if let Some(n) = far {
                n.search(p, best);
            }
        }
    }

    fn nearest(&self, p: [u8; 3]) -> u8 {
        let mut best = None;
        self.search(p, &mut best);
        best.expect("a tree has a node").1
    }
}

impl Converter for Climage {
    fn info(&self) -> &'static ConverterInfo {
        &INFO
    }

    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid {
        // _toAnsi: scale = width / 80; height // scale, trimmed to even.
        let scale = image.width() as f64 / columns as f64;
        let mut height = py_floordiv(image.height() as f64, scale) as u32;
        height -= height % 2;
        let height = height.max(2);
        // Image.open(...).convert("RGB"), then resize's default (bicubic).
        let img = resample(&drop_alpha(image), columns, height, Kernel::Bicubic);
        let rows = height / 2;
        let mut grid = Grid::new(columns as usize, rows as usize);
        // The "linuxconsole" system colors, codes 0–15 in SGR order.
        let tree = KdNode::create(VGA_SGR.iter().enumerate().map(|(i, &c)| (c, i as u8)).collect(), 0).expect("palette");
        let color = |x: u32, y: u32| {
            let p = img.get_pixel(x, y);
            Color::Ansi(tree.nearest([p[0], p[1], p[2]]))
        };
        for row in 0..rows {
            for x in 0..columns {
                grid.set(x as usize, row as usize, Cell::new(cp437::LOWER_HALF, color(x, row * 2 + 1), color(x, row * 2)));
            }
        }
        grid
    }
}
