//! The drawing tools and selection (docs/roadmap.md, Phase 1c): lines,
//! rectangles, box drawing, flood fill, the half-block brush, and the
//! selection's copy, cut, paste, move, fill, clear and flip.
//!
//! Every tool is an edit in a stroke (edit.rs), so one drag or one command
//! is one undo step. A shape being dragged is redrawn at each move by
//! retracting the stroke and drawing it again, so the preview is the real
//! art and letting go leaves exactly one step.

use icy_engine::{AttributeColor, AttributedChar, TextAttribute, TextPane};
use serde::{Deserialize, Serialize};

use crate::document::Document;
use crate::edit::{CellEdit, CellRect};

/// A cell position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// What a tool paints: a `None` field keeps what the cell has (the
/// "colors only" and "character only" paint modes).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pen {
    pub code: Option<u8>,
    pub fg: Option<u8>,
    pub bg: Option<u8>,
}

impl Pen {
    fn at(self, x: i32, y: i32) -> CellEdit {
        CellEdit { x, y, code: self.code, fg: self.fg, bg: self.bg }
    }
}

/// A shape dragged from one cell to another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Shape {
    Line { from: Point, to: Point },
    Rectangle { from: Point, to: Point, filled: bool },
    /// A box drawn with the single or double line characters. The pen's
    /// character is ignored; with no character (colors only) it recolors.
    Box { from: Point, to: Point, double: bool },
}

/// Something done to the selected rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SelectionOp {
    /// Blank spaces, light gray on black.
    Clear,
    Fill { pen: Pen },
    FlipHorizontal,
    FlipVertical,
    /// Moves the cells so the top left lands at `to`, leaving blanks.
    Move { to: Point },
}

/// Cells copied from a document, with every attribute (24-bit colors,
/// blink) kept, for pasting into this or another document.
#[derive(Debug, Clone, PartialEq)]
pub struct Clip {
    pub width: i32,
    pub height: i32,
    cells: Vec<AttributedChar>,
}

impl Clip {
    fn get(&self, x: i32, y: i32) -> AttributedChar {
        self.cells[(y * self.width + x) as usize]
    }
}

const BLANK_CODES: [char; 3] = ['\0', ' ', '\u{ff}'];

fn blank() -> AttributedChar {
    AttributedChar::new(' ', TextAttribute::new(7, 0))
}

/// A cell showing nothing but black, which a transparent paste skips.
fn is_empty(ch: AttributedChar) -> bool {
    BLANK_CODES.contains(&ch.ch) && matches!(ch.attribute.background_color(), AttributeColor::Palette(0) | AttributeColor::Transparent)
}

/// The rectangle with corners `a` and `b`, either way round.
pub fn rect_between(a: Point, b: Point) -> CellRect {
    CellRect { x: a.x.min(b.x), y: a.y.min(b.y), width: (a.x - b.x).abs() + 1, height: (a.y - b.y).abs() + 1 }
}

/// Every cell on the line from `a` to `b` (Bresenham), with no gaps.
pub fn line_points(a: Point, b: Point) -> Vec<Point> {
    let mut points = Vec::new();
    let (mut x, mut y) = (a.x, a.y);
    let dx = (b.x - x).abs();
    let dy = -(b.y - y).abs();
    let sx = if x < b.x { 1 } else { -1 };
    let sy = if y < b.y { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        points.push(Point { x, y });
        if x == b.x && y == b.y {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
    points
}

/// Box-drawing characters in CP437: horizontal, vertical, then the corners
/// top left, top right, bottom left, bottom right.
const SINGLE_BOX: [u8; 6] = [0xc4, 0xb3, 0xda, 0xbf, 0xc0, 0xd9];
const DOUBLE_BOX: [u8; 6] = [0xcd, 0xba, 0xc9, 0xbb, 0xc8, 0xbc];

impl Shape {
    /// The edits that draw the shape with `pen`.
    pub fn edits(self, pen: Pen) -> Vec<CellEdit> {
        match self {
            Shape::Line { from, to } => line_points(from, to).into_iter().map(|p| pen.at(p.x, p.y)).collect(),
            Shape::Rectangle { from, to, filled } => {
                let r = rect_between(from, to);
                let mut edits = Vec::new();
                for y in r.y..r.y + r.height {
                    for x in r.x..r.x + r.width {
                        let edge = y == r.y || y == r.y + r.height - 1 || x == r.x || x == r.x + r.width - 1;
                        if filled || edge {
                            edits.push(pen.at(x, y));
                        }
                    }
                }
                edits
            }
            Shape::Box { from, to, double } => {
                let [h, v, tl, tr, bl, br] = if double { DOUBLE_BOX } else { SINGLE_BOX };
                let r = rect_between(from, to);
                let (right, bottom) = (r.x + r.width - 1, r.y + r.height - 1);
                let mut edits = Vec::new();
                for y in r.y..=bottom {
                    for x in r.x..=right {
                        let code = if r.height == 1 {
                            h
                        } else if r.width == 1 {
                            v
                        } else if (x, y) == (r.x, r.y) {
                            tl
                        } else if (x, y) == (right, r.y) {
                            tr
                        } else if (x, y) == (r.x, bottom) {
                            bl
                        } else if (x, y) == (right, bottom) {
                            br
                        } else if y == r.y || y == bottom {
                            h
                        } else if x == r.x || x == right {
                            v
                        } else {
                            continue;
                        };
                        edits.push(CellEdit { x, y, code: pen.code.map(|_| code), fg: pen.fg, bg: pen.bg });
                    }
                }
                edits
            }
        }
    }
}

/// Characters that are each other's mirror image across a vertical line
/// (for flipping horizontally), in CP437.
const MIRROR_HORIZONTAL: &[(u8, u8)] = &[
    (0xdd, 0xde), // ▌ ▐
    (0xda, 0xbf), // ┌ ┐
    (0xc0, 0xd9), // └ ┘
    (0xc3, 0xb4), // ├ ┤
    (0xc9, 0xbb), // ╔ ╗
    (0xc8, 0xbc), // ╚ ╝
    (0xcc, 0xb9), // ╠ ╣
    (0xd5, 0xb8), // ╒ ╕
    (0xd6, 0xb7), // ╓ ╖
    (0xd4, 0xbe), // ╘ ╛
    (0xd3, 0xbd), // ╙ ╜
    (0xc6, 0xb5), // ╞ ╡
    (0xc7, 0xb6), // ╟ ╢
    (0x11, 0x10), // ◄ ►
    (0x1b, 0x1a), // ← →
    (0xae, 0xaf), // « »
    (b'(', b')'),
    (b'[', b']'),
    (b'{', b'}'),
    (b'<', b'>'),
    (b'/', b'\\'),
];

/// Mirror images across a horizontal line (for flipping vertically).
const MIRROR_VERTICAL: &[(u8, u8)] = &[
    (0xdf, 0xdc), // ▀ ▄
    (0xda, 0xc0), // ┌ └
    (0xbf, 0xd9), // ┐ ┘
    (0xc2, 0xc1), // ┬ ┴
    (0xc9, 0xc8), // ╔ ╚
    (0xbb, 0xbc), // ╗ ╝
    (0xcb, 0xca), // ╦ ╩
    (0xd5, 0xd4), // ╒ ╘
    (0xd6, 0xd3), // ╓ ╙
    (0xb8, 0xbe), // ╕ ╛
    (0xb7, 0xbd), // ╖ ╜
    (0xd1, 0xcf), // ╤ ╧
    (0xd2, 0xd0), // ╥ ╨
    (0x1e, 0x1f), // ▲ ▼
    (0x18, 0x19), // ↑ ↓
    (b'/', b'\\'),
];

fn mirror(ch: AttributedChar, pairs: &[(u8, u8)], pc_font: bool) -> AttributedChar {
    let code = ch.ch as u32;
    for &(a, b) in pairs {
        // Only ASCII pairs outside the PC fonts: the Amiga's upper half isn't CP437.
        if !pc_font && (a > 0x7f || b > 0x7f || a < 0x20) {
            continue;
        }
        let swap = if code == u32::from(a) {
            b
        } else if code == u32::from(b) {
            a
        } else {
            continue;
        };
        return AttributedChar { ch: char::from(swap), ..ch };
    }
    ch
}

/// The half-block brush's two "pixels" in a cell: the colors of its top and
/// bottom halves, if the cell reads as half-blocks (▀ ▄ █ or a blank).
fn halves(ch: AttributedChar) -> Option<(u8, u8)> {
    let fg = palette(ch.attribute.foreground_color());
    let bg = palette(ch.attribute.background_color());
    match ch.ch as u32 {
        0xdf => Some((fg, bg)),
        0xdc => Some((bg, fg)),
        0xdb => Some((fg, fg)),
        0x00 | 0x20 | 0xff => Some((bg, bg)),
        _ => None,
    }
}

fn palette(color: AttributeColor) -> u8 {
    match color {
        AttributeColor::Palette(n) | AttributeColor::ExtendedPalette(n) => n & 0x0f,
        AttributeColor::Rgb(r, g, b) => icy_engine::nearest_dos_color((r, g, b)) as u8,
        AttributeColor::Transparent => 0,
    }
}

/// The cell showing `top` over `bottom` in half-blocks. Without iCE a
/// background can only be one of the first eight colors, so the bright one
/// goes in the foreground; with two bright colors the bottom darkens.
fn half_block_cell(top: u8, bottom: u8, ice: bool) -> (u8, u8, u8) {
    let can_bg = |c: u8| ice || c < 8;
    if top == bottom {
        (0xdb, top, if can_bg(top) { top } else { 0 })
    } else if can_bg(bottom) {
        (0xdf, top, bottom)
    } else if can_bg(top) {
        (0xdc, bottom, top)
    } else {
        (0xdf, top, bottom & 7)
    }
}

impl Document {
    fn pc_font(&self) -> bool {
        !self.buffer.font(0).is_some_and(|f| f.name().starts_with("Amiga"))
    }

    /// The rectangle cut to the canvas, or `None` if nothing's left.
    fn clamp(&self, rect: CellRect) -> Option<CellRect> {
        let x = rect.x.max(0);
        let y = rect.y.max(0);
        let right = (rect.x + rect.width).min(self.buffer.width());
        let bottom = (rect.y + rect.height).min(self.rows);
        (right > x && bottom > y).then(|| CellRect { x, y, width: right - x, height: bottom - y })
    }

    fn char_at(&self, x: i32, y: i32) -> AttributedChar {
        self.buffer.layers[self.buffer.layers.len() - 1].char_at((x, y).into())
    }

    /// Draws `shape` as stroke `stroke`, replacing what the same stroke drew
    /// before: call it at each move of a drag, with one stroke id per drag.
    /// Returns the cells to redraw (what was drawn before, and now).
    pub fn draw_shape(&mut self, stroke: u32, shape: Shape, pen: Pen) -> Option<CellRect> {
        let old = self.retract(stroke);
        let new = self.apply(stroke, &shape.edits(pen));
        union(old, new)
    }

    /// Flood fill from a cell: the cell and every cell joined to it (up,
    /// down, left, right) that holds exactly the same character and colors.
    pub fn flood_fill(&mut self, stroke: u32, at: Point, pen: Pen) -> Option<CellRect> {
        let (columns, rows) = (self.buffer.width(), self.rows);
        if at.x < 0 || at.y < 0 || at.x >= columns || at.y >= rows {
            return None;
        }
        let target = self.char_at(at.x, at.y);
        let mut seen = vec![false; (columns * rows) as usize];
        let mut stack = vec![at];
        let mut edits = Vec::new();
        seen[(at.y * columns + at.x) as usize] = true;
        while let Some(p) = stack.pop() {
            edits.push(pen.at(p.x, p.y));
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (x, y) = (p.x + dx, p.y + dy);
                if x < 0 || y < 0 || x >= columns || y >= rows {
                    continue;
                }
                let i = (y * columns + x) as usize;
                if !seen[i] && self.char_at(x, y) == target {
                    seen[i] = true;
                    stack.push(Point { x, y });
                }
            }
        }
        self.apply(stroke, &edits)
    }

    /// The half-block brush: paints `color` into half-cell "pixels" along
    /// the line from `from` to `to`, where a point's `y` counts half rows
    /// (so the canvas is columns × 2·rows pixels). A cell that isn't
    /// half-blocks already becomes one, its other half the background.
    pub fn half_block(&mut self, stroke: u32, from: Point, to: Point, color: u8) -> Option<CellRect> {
        let color = color & 0x0f;
        let ice = self.ice;
        let mut dirty = None;
        for p in line_points(from, to) {
            let (x, y, top_half) = (p.x, p.y.div_euclid(2), p.y.rem_euclid(2) == 0);
            if p.y < 0 {
                continue;
            }
            let changed = self.commit(
                stroke,
                [(x, y, move |before: AttributedChar| {
                    let bg = palette(before.attribute.background_color());
                    let (top, bottom) = halves(before).unwrap_or((bg, bg));
                    let (top, bottom) = if top_half { (color, bottom) } else { (top, color) };
                    let (code, fg, bg) = half_block_cell(top, bottom, ice);
                    let mut after = before;
                    after.ch = char::from(code);
                    after.attribute = TextAttribute::new(u32::from(fg), u32::from(bg));
                    after
                })],
            );
            dirty = union(dirty, changed);
        }
        dirty
    }

    /// Copies the cells in `rect` (cut to the canvas).
    pub fn copy(&self, rect: CellRect) -> Option<Clip> {
        let r = self.clamp(rect)?;
        let cells = (r.y..r.y + r.height).flat_map(|y| (r.x..r.x + r.width).map(move |x| (x, y))).map(|(x, y)| self.char_at(x, y)).collect();
        Some(Clip { width: r.width, height: r.height, cells })
    }

    /// Pastes `clip` with its top left at `at`. A transparent paste leaves
    /// the art showing through the clip's empty cells (blanks on black).
    /// Returns the cells to redraw.
    pub fn paste(&mut self, stroke: u32, clip: &Clip, at: Point, transparent: bool) -> Option<CellRect> {
        let cells = (0..clip.height)
            .flat_map(|y| (0..clip.width).map(move |x| (x, y)))
            .map(|(x, y)| (x, y, clip.get(x, y)))
            .filter(|&(_, _, ch)| !transparent || !is_empty(ch))
            .map(|(x, y, ch)| (at.x + x, at.y + y, move |_| ch))
            .collect::<Vec<_>>();
        self.commit(stroke, cells)
    }

    /// Does `op` to the cells in `rect`. Returns the cells to redraw.
    pub fn selection(&mut self, stroke: u32, rect: CellRect, op: SelectionOp) -> Option<CellRect> {
        let r = self.clamp(rect)?;
        let cells = || (r.y..r.y + r.height).flat_map(move |y| (r.x..r.x + r.width).map(move |x| (x, y)));
        match op {
            SelectionOp::Clear => self.commit(stroke, cells().map(|(x, y)| (x, y, |_| blank()))),
            SelectionOp::Fill { pen } => self.apply(stroke, &cells().map(|(x, y)| pen.at(x, y)).collect::<Vec<_>>()),
            SelectionOp::FlipHorizontal | SelectionOp::FlipVertical => {
                let clip = self.copy(r)?;
                let horizontal = op == SelectionOp::FlipHorizontal;
                let pairs = if horizontal { MIRROR_HORIZONTAL } else { MIRROR_VERTICAL };
                let pc = self.pc_font();
                let flipped: Vec<_> = cells()
                    .map(|(x, y)| {
                        let (cx, cy) = (x - r.x, y - r.y);
                        let ch = if horizontal { clip.get(clip.width - 1 - cx, cy) } else { clip.get(cx, clip.height - 1 - cy) };
                        let ch = mirror(ch, pairs, pc);
                        (x, y, move |_| ch)
                    })
                    .collect();
                self.commit(stroke, flipped)
            }
            SelectionOp::Move { to } => {
                let clip = self.copy(r)?;
                let cleared = self.selection(stroke, r, SelectionOp::Clear);
                let pasted = self.paste(stroke, &clip, to, false);
                union(cleared, pasted)
            }
        }
    }
}

fn union(a: Option<CellRect>, b: Option<CellRect>) -> Option<CellRect> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.union(b)),
        (a, None) => a,
        (None, b) => b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit::CellInfo;

    const PEN: Pen = Pen { code: Some(b'#'), fg: Some(14), bg: Some(1) };

    fn p(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    fn codes(doc: &Document, y: i32) -> String {
        // decode_cp437 trims trailing blanks, so one cell at a time, blanks kept.
        (0..doc.info().columns)
            .map(|x| match doc.cell(x, y).unwrap().code as u8 {
                b' ' => ' ',
                c => crate::decode_cp437(&[c]).chars().next().unwrap(),
            })
            .collect()
    }

    #[test]
    fn a_dragged_line_is_one_undo_step_at_its_last_position() {
        let mut doc = Document::new_blank(10, 5, true).unwrap();
        doc.draw_shape(1, Shape::Line { from: p(0, 0), to: p(9, 0) }, PEN);
        // The drag moves on: the first line goes, the new one is drawn.
        let dirty = doc.draw_shape(1, Shape::Line { from: p(0, 0), to: p(3, 3) }, PEN).unwrap();
        assert_eq!(dirty, CellRect { x: 0, y: 0, width: 10, height: 4 });
        assert_eq!(codes(&doc, 0), "#         ");
        assert_eq!(doc.cell(3, 3).unwrap().code, u32::from(b'#'));
        doc.undo();
        assert!(!doc.can_undo() && !doc.is_edited());
        assert_eq!(doc.cell(3, 3).unwrap().code, 32);
    }

    #[test]
    fn a_new_drag_doesnt_take_back_the_last() {
        let mut doc = Document::new_blank(10, 5, true).unwrap();
        doc.draw_shape(1, Shape::Line { from: p(0, 0), to: p(9, 0) }, PEN);
        doc.draw_shape(2, Shape::Line { from: p(0, 1), to: p(9, 1) }, PEN);
        assert_eq!(codes(&doc, 0), "##########");
        assert_eq!(codes(&doc, 1), "##########");
        doc.undo();
        doc.redo();
        // Each drag has its own stroke id, so the next one leaves both.
        doc.draw_shape(3, Shape::Line { from: p(0, 2), to: p(1, 2) }, PEN);
        assert_eq!(codes(&doc, 1), "##########");
        assert_eq!(codes(&doc, 2), "##        ");
    }

    #[test]
    fn rectangles_outline_or_filled_either_way_round() {
        let mut doc = Document::new_blank(6, 4, true).unwrap();
        doc.draw_shape(1, Shape::Rectangle { from: p(4, 3), to: p(1, 0), filled: false }, PEN);
        assert_eq!(codes(&doc, 0), " #### ");
        assert_eq!(codes(&doc, 1), " #  # ");
        assert_eq!(codes(&doc, 3), " #### ");
        doc.draw_shape(2, Shape::Rectangle { from: p(0, 1), to: p(5, 2), filled: true }, Pen { code: Some(b'o'), ..PEN });
        assert_eq!(codes(&doc, 1), "oooooo");
    }

    #[test]
    fn boxes_use_the_line_characters() {
        let mut doc = Document::new_blank(5, 4, true).unwrap();
        doc.draw_shape(1, Shape::Box { from: p(0, 0), to: p(4, 2), double: false }, PEN);
        assert_eq!(codes(&doc, 0), "┌───┐");
        assert_eq!(codes(&doc, 1), "│   │");
        assert_eq!(codes(&doc, 2), "└───┘");
        doc.draw_shape(2, Shape::Box { from: p(0, 3), to: p(4, 3), double: true }, PEN);
        assert_eq!(codes(&doc, 3), "═════");
        assert_eq!(doc.cell(0, 0).unwrap().fg, 14);
        // Colors only: recolors the outline, keeps the characters.
        doc.draw_shape(3, Shape::Box { from: p(0, 0), to: p(4, 2), double: true }, Pen { code: None, fg: Some(12), bg: None });
        assert_eq!(codes(&doc, 0), "┌───┐");
        assert_eq!(doc.cell(2, 2).unwrap().fg, 12);
    }

    #[test]
    fn flood_fill_stops_at_different_cells() {
        let mut doc = Document::new_blank(6, 3, true).unwrap();
        doc.draw_shape(1, Shape::Line { from: p(3, 0), to: p(3, 2) }, PEN);
        doc.flood_fill(2, p(0, 0), Pen { code: Some(b'.'), fg: Some(2), bg: Some(0) });
        assert_eq!(codes(&doc, 1), "...#  ");
        doc.undo();
        assert_eq!(codes(&doc, 1), "   #  ");
        // Filling with what's there already changes nothing.
        assert_eq!(doc.flood_fill(3, p(0, 0), Pen { code: Some(b' '), fg: Some(7), bg: Some(0) }), None);
    }

    #[test]
    fn the_half_block_brush_paints_half_cells() {
        let mut doc = Document::new_blank(4, 2, true).unwrap();
        // Top half of cell (0, 0) red: ▀ red on black.
        doc.half_block(1, p(0, 0), p(0, 0), 4);
        assert_eq!(doc.cell(0, 0), Some(CellInfo { code: 0xdf, fg: 4, bg: 0, blink: false, truecolor: false }));
        // Bottom half red too: a full block.
        doc.half_block(1, p(0, 1), p(0, 1), 4);
        assert_eq!(doc.cell(0, 0).unwrap().code, 0xdb);
        // Bottom half yellow: ▀ red on yellow with iCE.
        doc.half_block(1, p(0, 1), p(0, 1), 14);
        assert_eq!(doc.cell(0, 0), Some(CellInfo { code: 0xdf, fg: 4, bg: 14, blink: false, truecolor: false }));
        // A vertical line of pixels across two cells, one stroke.
        doc.half_block(2, p(3, 0), p(3, 3), 2);
        assert_eq!(doc.cell(3, 0).unwrap().code, 0xdb);
        assert_eq!(doc.cell(3, 1).unwrap().code, 0xdb);
        doc.undo();
        assert_eq!(doc.cell(3, 1).unwrap().code, 32);
    }

    #[test]
    fn without_ice_the_bright_half_goes_in_the_foreground() {
        let mut doc = Document::new_blank(1, 1, false).unwrap();
        doc.half_block(1, p(0, 1), p(0, 1), 14);
        assert_eq!(doc.cell(0, 0), Some(CellInfo { code: 0xdc, fg: 14, bg: 0, blink: false, truecolor: false }));
    }

    #[test]
    fn copy_and_paste_keep_every_cell() {
        let mut doc = Document::new_blank(6, 3, true).unwrap();
        doc.apply(1, &[CellEdit { x: 0, y: 0, code: Some(b'A'), fg: Some(9), bg: Some(4) }, CellEdit { x: 1, y: 0, code: Some(b'B'), fg: Some(3), bg: Some(0) }]);
        let clip = doc.copy(CellRect { x: 0, y: 0, width: 2, height: 2 }).unwrap();
        assert_eq!((clip.width, clip.height), (2, 2));
        let dirty = doc.paste(2, &clip, p(3, 1), false).unwrap();
        assert_eq!(dirty, CellRect { x: 3, y: 1, width: 2, height: 1 });
        assert_eq!(doc.cell(3, 1), doc.cell(0, 0));
        assert_eq!(codes(&doc, 1), "   AB ");
        // Off the edge: what fits.
        doc.paste(3, &clip, p(5, 2), false);
        assert_eq!(codes(&doc, 2), "     A");
        assert!(doc.copy(CellRect { x: 10, y: 10, width: 2, height: 2 }).is_none());
    }

    #[test]
    fn a_transparent_paste_skips_empty_cells() {
        let mut doc = Document::new_blank(4, 1, true).unwrap();
        doc.apply(1, &[CellEdit { x: 0, y: 0, code: Some(b'X'), fg: Some(7), bg: Some(0) }]);
        let clip = doc.copy(CellRect { x: 0, y: 0, width: 2, height: 1 }).unwrap();
        doc.apply(2, &[CellEdit { x: 3, y: 0, code: Some(b'Z'), fg: Some(7), bg: Some(0) }]);
        doc.paste(3, &clip, p(2, 0), true);
        assert_eq!(codes(&doc, 0), "X XZ");
        doc.paste(4, &clip, p(2, 0), false);
        assert_eq!(codes(&doc, 0), "X X ");
    }

    #[test]
    fn flips_mirror_the_characters_too() {
        let mut doc = Document::new_blank(4, 2, true).unwrap();
        doc.draw_shape(1, Shape::Box { from: p(0, 0), to: p(2, 1), double: false }, PEN);
        doc.apply(2, &[CellEdit { x: 3, y: 0, code: Some(0xdf), fg: None, bg: None }]);
        let all = CellRect { x: 0, y: 0, width: 4, height: 2 };
        doc.selection(3, all, SelectionOp::FlipHorizontal);
        assert_eq!(codes(&doc, 0), "▀┌─┐");
        doc.selection(4, all, SelectionOp::FlipVertical);
        assert_eq!(codes(&doc, 0), " ┌─┐");
        assert_eq!(codes(&doc, 1), "▄└─┘");
        doc.undo();
        doc.undo();
        assert_eq!(codes(&doc, 0), "┌─┐▀");
    }

    #[test]
    fn moving_leaves_blanks_and_is_one_step() {
        let mut doc = Document::new_blank(5, 1, true).unwrap();
        doc.apply(1, &[CellEdit { x: 0, y: 0, code: Some(b'a'), fg: Some(7), bg: Some(0) }, CellEdit { x: 1, y: 0, code: Some(b'b'), fg: Some(7), bg: Some(0) }]);
        // Overlapping the old place.
        doc.selection(2, CellRect { x: 0, y: 0, width: 2, height: 1 }, SelectionOp::Move { to: p(1, 0) });
        assert_eq!(codes(&doc, 0), " ab  ");
        doc.undo();
        assert_eq!(codes(&doc, 0), "ab   ");
        doc.redo();
        doc.selection(3, CellRect { x: 1, y: 0, width: 2, height: 1 }, SelectionOp::Clear);
        assert_eq!(codes(&doc, 0), "     ");
        doc.selection(4, CellRect { x: 0, y: 0, width: 9, height: 9 }, SelectionOp::Fill { pen: PEN });
        assert_eq!(codes(&doc, 0), "#####");
    }
}
