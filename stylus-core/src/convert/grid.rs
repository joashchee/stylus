//! What every image converter produces: a grid of character cells (a CP437
//! byte, a foreground and a background color), and the one writer that
//! turns a grid into an .ANS file with a SAUCE record.
//!
//! Converters decide the cells; how the escape codes are spelled is the
//! writer's job, so every converter's file is laid out the same way and a
//! difference between two files is always a difference in the art.

/// A cell color, in whatever color space the converter chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// The terminal's default color (SGR 39/49): light gray on black in
    /// ANSI art viewers. Converters that "leave a pixel transparent" use it.
    Default,
    /// One of the 16 ANSI colors in SGR order (0 black, 1 red, 2 green,
    /// 3 yellow, 4 blue, 5 magenta, 6 cyan, 7 white, 8–15 bright). Written as
    /// SGR 30–37/40–47, bright foregrounds with bold and bright backgrounds
    /// with blink under iCE colors, as in scene ANSI.
    Ansi(u8),
    /// An xterm 256-color index (SGR 38;5 / 48;5).
    Xterm(u8),
    /// 24-bit color (SGR 38;2 / 48;2).
    Rgb(u8, u8, u8),
}

impl Color {
    /// An ANSI color given in DOS/VGA attribute order (0 black, 1 blue,
    /// 2 green, 3 cyan, 4 red, 5 magenta, 6 brown, 7 gray, +8 bright), as
    /// BIN files and many scene tools number them.
    pub fn dos(index: u8) -> Color {
        const TO_SGR: [u8; 8] = [0, 4, 2, 6, 1, 5, 3, 7];
        Color::Ansi(TO_SGR[(index & 7) as usize] | (index & 8))
    }
}

/// One character cell. `ch` is a CP437 byte; control codes (below 0x20 and
/// 0x7F) aren't allowed, since an .ANS file can't show them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: u8,
    pub fg: Color,
    pub bg: Color,
}

impl Cell {
    pub const BLANK: Cell = Cell { ch: b' ', fg: Color::Default, bg: Color::Default };

    pub fn new(ch: u8, fg: Color, bg: Color) -> Cell {
        Cell { ch, fg, bg }
    }
}

/// CP437 bytes for the Unicode block and shade glyphs converters use.
pub mod cp437 {
    pub const LIGHT_SHADE: u8 = 0xB0; // ░
    pub const MEDIUM_SHADE: u8 = 0xB1; // ▒
    pub const DARK_SHADE: u8 = 0xB2; // ▓
    pub const FULL_BLOCK: u8 = 0xDB; // █
    pub const LOWER_HALF: u8 = 0xDC; // ▄
    pub const LEFT_HALF: u8 = 0xDD; // ▌
    pub const RIGHT_HALF: u8 = 0xDE; // ▐
    pub const UPPER_HALF: u8 = 0xDF; // ▀

}

#[derive(Debug, Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    /// Row-major, `width * height` cells.
    pub cells: Vec<Cell>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Grid {
        Grid {
            width,
            height,
            cells: vec![Cell::BLANK; width * height],
        }
    }

    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        let w = self.width;
        self.cells[y * w + x] = cell;
    }

    pub fn get(&self, x: usize, y: usize) -> Cell {
        self.cells[y * self.width + x]
    }

    /// Whether any cell has a bright ANSI background, which the file can
    /// only show with iCE colors.
    pub fn needs_ice(&self) -> bool {
        self.cells.iter().any(|c| matches!(c.bg, Color::Ansi(i) if i >= 8))
    }
}

/// What goes in the SAUCE record besides the grid's size.
pub struct AnsMeta<'a> {
    /// The title field (35 characters, cut to fit).
    pub title: &'a str,
    /// Date as `CCYYMMDD`.
    pub date: &'a str,
    /// SAUCE comment lines, each cut to 64 characters.
    pub comments: &'a [String],
}

/// Writes the grid as an .ANS file: CP437 text with SGR color codes, EOF
/// (0x1A) and one SAUCE record with the width, height, iCE flag and
/// comments.
///
/// There are no line breaks: every row fills the SAUCE width, and viewers
/// (icy_engine, ansilove) move to the next line after the last column by
/// themselves, so a CR LF there would add a blank row. Scene editors write
/// full rows the same way. Grids are at most 80 columns, the width viewers
/// assume without SAUCE.
pub fn write_ans(grid: &Grid, meta: &AnsMeta) -> Vec<u8> {
    debug_assert!(grid.width <= 80);
    let mut out = Vec::with_capacity(grid.cells.len() * 4);
    let mut state = Sgr::reset();
    out.extend_from_slice(b"\x1b[0m");
    for y in 0..grid.height {
        for x in 0..grid.width {
            let cell = grid.get(x, y);
            debug_assert!(cell.ch >= 0x20 && cell.ch != 0x7f, "control code in a converted cell");
            let ch = if cell.ch < 0x20 || cell.ch == 0x7f { b' ' } else { cell.ch };
            state.change_to(cell.fg, cell.bg, &mut out);
            out.push(ch);
        }
    }
    out.extend_from_slice(b"\x1b[0m");
    let data_size = out.len() as u32;
    out.push(0x1a);
    out.extend(sauce(grid, meta, data_size));
    out
}

/// The SGR state the writer has set.
struct Sgr {
    fg: Color,
    bg: Color,
    bold: bool,
    blink: bool,
}

impl Sgr {
    fn reset() -> Sgr {
        Sgr {
            fg: Color::Default,
            bg: Color::Default,
            bold: false,
            blink: false,
        }
    }

    fn change_to(&mut self, fg: Color, bg: Color, out: &mut Vec<u8>) {
        let bold = matches!(fg, Color::Ansi(i) if i >= 8);
        let blink = matches!(bg, Color::Ansi(i) if i >= 8);
        if fg == self.fg && bg == self.bg && bold == self.bold && blink == self.blink {
            return;
        }
        let mut params: Vec<String> = Vec::new();
        // Bold and blink can only be switched off with a full reset.
        if (self.bold && !bold) || (self.blink && !blink) {
            params.push("0".into());
            *self = Sgr::reset();
        }
        if bold && !self.bold {
            params.push("1".into());
        }
        if blink && !self.blink {
            params.push("5".into());
        }
        if fg != self.fg {
            params.push(color_param(fg, false));
        }
        if bg != self.bg {
            params.push(color_param(bg, true));
        }
        *self = Sgr { fg, bg, bold, blink };
        if !params.is_empty() {
            out.extend_from_slice(format!("\x1b[{}m", params.join(";")).as_bytes());
        }
    }
}

fn color_param(color: Color, background: bool) -> String {
    let base = if background { 40 } else { 30 };
    match color {
        Color::Default => (base + 9).to_string(),
        Color::Ansi(i) => (base + (i & 7) as u32).to_string(),
        Color::Xterm(i) => format!("{};5;{i}", base + 8),
        Color::Rgb(r, g, b) => format!("{};2;{r};{g};{b}", base + 8),
    }
}

fn sauce(grid: &Grid, meta: &AnsMeta, data_size: u32) -> Vec<u8> {
    fn field(s: &str, len: usize) -> Vec<u8> {
        let mut v: Vec<u8> = s.chars().map(|c| if (' '..='~').contains(&c) { c as u8 } else { b'?' }).take(len).collect();
        v.resize(len, b' ');
        v
    }
    let comments: Vec<&String> = meta.comments.iter().take(255).collect();
    let mut out = Vec::new();
    if !comments.is_empty() {
        out.extend_from_slice(b"COMNT");
        for line in &comments {
            out.extend(field(line, 64));
        }
    }
    out.extend_from_slice(b"SAUCE00");
    out.extend(field(meta.title, 35));
    out.extend(field("", 20)); // author
    out.extend(field("", 20)); // group
    out.extend(field(meta.date, 8));
    out.extend(data_size.to_le_bytes());
    out.push(1); // data type: character
    out.push(1); // file type: ANSi
    out.extend((grid.width as u16).to_le_bytes());
    out.extend((grid.height.min(u16::MAX as usize) as u16).to_le_bytes());
    out.extend([0u8; 4]); // tinfo3, tinfo4
    out.push(comments.len() as u8);
    out.push(u8::from(grid.needs_ice())); // flags: bit 0, iCE colors
    let mut font = b"IBM VGA".to_vec(); // font name, NUL-padded
    font.resize(22, 0);
    out.extend(font);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;

    fn meta() -> AnsMeta<'static> {
        AnsMeta { title: "t", date: "20260929", comments: &[] }
    }

    /// Renders one cell's pixels (8×16) as RGBA.
    fn cell_pixels(doc: &Document, x: usize, y: usize) -> Vec<[u8; 4]> {
        let band = doc.render_rows(y as i32, 1, true);
        let w = band.width as usize;
        let cw = w / doc.info().columns as usize;
        (0..16).flat_map(|py| (0..cw).map(move |px| (py, px))).map(|(py, px)| {
            let o = (py * w + x * cw + px) * 4;
            [band.rgba[o], band.rgba[o + 1], band.rgba[o + 2], band.rgba[o + 3]]
        }).collect()
    }

    #[test]
    fn full_width_rows_dont_wrap_twice() {
        let mut g = Grid::new(80, 3);
        for y in 0..3 {
            for x in 0..80 {
                g.set(x, y, Cell::new(b'#', Color::Ansi((x % 16) as u8), Color::Ansi(((x + y) % 8) as u8)));
            }
        }
        let doc = Document::open("ans", &write_ans(&g, &meta())).unwrap();
        assert_eq!(doc.info().rows, 3);
        let mut narrow = Grid::new(40, 3);
        narrow.set(0, 2, Cell::new(b'#', Color::Ansi(7), Color::Ansi(0)));
        let doc_narrow = Document::open("ans", &write_ans(&narrow, &meta())).unwrap();
        assert_eq!((doc_narrow.info().columns, doc_narrow.info().rows), (40, 3));
        assert_eq!(doc.info().columns, 80);
    }

    #[test]
    fn colors_render_as_written() {
        let mut g = Grid::new(80, 1);
        g.set(0, 0, Cell::new(b' ', Color::Default, Color::Rgb(12, 34, 56)));
        g.set(1, 0, Cell::new(b' ', Color::Default, Color::Xterm(196)));
        g.set(2, 0, Cell::new(b' ', Color::Default, Color::Ansi(12)));
        g.set(3, 0, Cell::new(cp437::FULL_BLOCK, Color::Ansi(14), Color::Ansi(0)));
        g.set(4, 0, Cell::new(cp437::FULL_BLOCK, Color::Xterm(21), Color::Ansi(0)));
        g.set(5, 0, Cell::new(cp437::FULL_BLOCK, Color::Rgb(200, 100, 0), Color::Ansi(0)));
        g.set(6, 0, Cell::new(cp437::FULL_BLOCK, Color::Ansi(1), Color::Ansi(0)));
        let ans = write_ans(&g, &meta());
        let doc = Document::open("ans", &ans).unwrap();
        assert!(doc.settings().ice_colors, "bright background sets the iCE flag");
        let px = |x| cell_pixels(&doc, x, 0)[0];
        assert_eq!(px(0), [12, 34, 56, 255]);
        assert_eq!(px(1), [255, 0, 0, 255]);
        assert_eq!(px(2), [85, 85, 255, 255]);
        assert_eq!(px(3), [85, 255, 255, 255]);
        assert_eq!(px(4), [0, 0, 255, 255]);
        assert_eq!(px(5), [200, 100, 0, 255]);
        assert_eq!(px(6), [170, 0, 0, 255], "normal red after bright cyan: bold was reset");
    }
}
