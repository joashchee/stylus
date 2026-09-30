//! Editing a document: cell edits in undoable strokes, resizing, and saving
//! through icy_engine's format writers with a fresh SAUCE record.
//!
//! Stylus keeps its own editing layer over icy_engine's `TextBuffer` rather
//! than using `icy_engine_edit`, which brings a GUI toolkit, tokio and
//! websockets (its collaboration server) that neither the web build nor
//! CLAUDE.md rule 1 allows (docs/roadmap.md, Phase 1b).
//!
//! Edits go to the top layer. Colors are palette indexes (0–15); with iCE
//! off, backgrounds are 0–7, as the format stores them.

use icy_engine::formats::{AnsiCompatibilityLevel, FileFormat, FormatCapabilities, FormatOptions, IssueType, SaveOptions, SauceMetaData};
use icy_engine::{AttributeColor, AttributedChar, IceMode, Layer, Size, TextAttribute, TextPane};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::document::Document;

/// A rectangle of cells, for what an edit changed (to redraw) and what to render.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl CellRect {
    pub(crate) fn cell(x: i32, y: i32) -> Self {
        CellRect { x, y, width: 1, height: 1 }
    }

    pub(crate) fn union(self, other: CellRect) -> CellRect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = (self.x + self.width).max(other.x + other.width);
        let bottom = (self.y + self.height).max(other.y + other.height);
        CellRect { x, y, width: right - x, height: bottom - y }
    }
}

/// One cell to change. A `None` field keeps what the cell has, so the same
/// edit paints character and colors, colors only, or the character only.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellEdit {
    pub x: i32,
    pub y: i32,
    /// The character code in the document's font (CP437 for the IBM fonts).
    pub code: Option<u8>,
    pub fg: Option<u8>,
    pub bg: Option<u8>,
}

/// What a cell holds, for the eyedropper and the status bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellInfo {
    pub code: u32,
    /// Palette index; for a 24-bit color, the nearest of the 16.
    pub fg: u8,
    pub bg: u8,
    pub blink: bool,
    /// Either color is 24-bit, so `fg`/`bg` are approximations.
    pub truecolor: bool,
}

/// The SAUCE text fields a save writes. The rest of the record (size, flags,
/// font) comes from the document itself.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SauceFields {
    pub title: String,
    pub author: String,
    pub group: String,
    /// CCYYMMDD. The file's own date when it had one, otherwise today.
    pub date: String,
    pub comments: Vec<String>,
}

/// Something a save in a given format would lose, in words for the user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLoss {
    /// The save can't go ahead at all (e.g. BIN wider than 255 columns).
    pub blocking: bool,
    pub message: String,
}

/// A format the editor saves, for the Save As dialog.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveFormat {
    pub extension: &'static str,
    pub name: &'static str,
}

/// Formats Phase 1 saves (docs/roadmap.md, 1c), all written by icy_engine.
pub const SAVE_FORMATS: &[SaveFormat] = &[
    SaveFormat { extension: "ans", name: "ANSI" },
    SaveFormat { extension: "asc", name: "Plain text (ASCII)" },
    SaveFormat { extension: "xb", name: "XBin" },
    SaveFormat { extension: "bin", name: "Binary text (BIN)" },
    SaveFormat { extension: "adf", name: "Artworx (ADF)" },
    SaveFormat { extension: "idf", name: "iCE Draw (IDF)" },
    SaveFormat { extension: "tnd", name: "TundraDraw (TND)" },
];

/// Fonts a text file without SAUCE can be shown in: the PC's, or the
/// Amiga's for Amiga ASCII. Names as SAUCE writes them.
pub const TEXT_FONTS: &[&str] = &[
    "IBM VGA",
    "Amiga Topaz 1",
    "Amiga Topaz 1+",
    "Amiga Topaz 2",
    "Amiga Topaz 2+",
    "Amiga P0T-NOoDLE",
    "Amiga MicroKnight",
    "Amiga MicroKnight+",
    "Amiga mOsOul",
];

/// One undo step.
#[derive(Clone)]
pub(crate) enum Step {
    /// Every cell a stroke changed: position, before, after. A stroke is all
    /// the edits sent with one stroke id (one drag of the pencil, one typed
    /// character), so undo takes the whole stroke back.
    Cells { stroke: u32, layer: usize, cells: Vec<(i32, i32, AttributedChar, AttributedChar)> },
    /// A resize, with every layer before and after.
    Resize { before: (Size, i32, Vec<Layer>), after: (Size, i32, Vec<Layer>) },
}

/// Undo and redo, each step with the revision it made. The document's
/// revision is the top undo step's (0 with none), so undoing back to the
/// last save makes the document unedited again.
#[derive(Default)]
pub(crate) struct History {
    pub undo: Vec<(i64, Step)>,
    pub redo: Vec<(i64, Step)>,
    /// Revision at the last save (0: as opened).
    pub saved: i64,
    /// The last revision handed out.
    pub last: i64,
}

impl History {
    fn revision(&self) -> i64 {
        self.undo.last().map_or(0, |(r, _)| *r)
    }
}

/// Largest canvas the editor makes or grows to, in cells.
pub const MAX_COLUMNS: i32 = 2000;
pub const MAX_ROWS: i32 = 100_000;

impl Document {
    /// A new, empty document: spaces, light gray on black, in IBM VGA.
    pub fn new_blank(columns: i32, rows: i32, ice_colors: bool) -> Result<Self, String> {
        if !(1..=MAX_COLUMNS).contains(&columns) || !(1..=MAX_ROWS).contains(&rows) {
            return Err(format!("A canvas can be 1 to {MAX_COLUMNS} columns and 1 to {MAX_ROWS} rows"));
        }
        let mut buffer = icy_engine::TextBuffer::create((columns, rows));
        let blank = AttributedChar::new(' ', TextAttribute::new(7, 0));
        for y in 0..rows {
            for x in 0..columns {
                buffer.layers[0].set_char((x, y), blank);
            }
        }
        buffer.ice_mode = if ice_colors { IceMode::Ice } else { IceMode::Blink };
        Ok(Document::from_buffer(buffer, None, "ANSI", rows, ice_colors))
    }

    fn edit_layer(&self) -> usize {
        self.buffer.layers.len().saturating_sub(1)
    }

    pub fn can_undo(&self) -> bool {
        !self.history.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.history.redo.is_empty()
    }

    /// Changed since it was opened or last saved.
    pub fn is_edited(&self) -> bool {
        self.history.revision() != self.history.saved
    }

    /// Records that the document was just saved as it is now.
    pub fn mark_saved(&mut self) {
        self.history.saved = self.history.revision();
    }

    pub fn cell(&self, x: i32, y: i32) -> Option<CellInfo> {
        if x < 0 || y < 0 || x >= self.buffer.width() || y >= self.rows {
            return None;
        }
        let ch = self.buffer.char_at((x, y).into());
        let fg = ch.attribute.foreground_color();
        let bg = ch.attribute.background_color();
        Some(CellInfo {
            code: ch.ch as u32,
            fg: palette_index(fg),
            bg: palette_index(bg),
            blink: ch.attribute.is_blinking(),
            truecolor: matches!(fg, AttributeColor::Rgb(..)) || matches!(bg, AttributeColor::Rgb(..)),
        })
    }

    /// Applies edits as part of stroke `stroke`: edits sent with the stroke id
    /// of the last undo step join it. Returns the cells that changed, to
    /// redraw, or `None` if nothing did.
    pub fn apply(&mut self, stroke: u32, edits: &[CellEdit]) -> Option<CellRect> {
        let ice = self.ice;
        self.commit(stroke, edits.iter().map(|edit| {
            let edit = *edit;
            (edit.x, edit.y, move |before: AttributedChar| {
                let mut after = before;
                if let Some(code) = edit.code {
                    after.ch = char::from(code);
                }
                if let Some(fg) = edit.fg {
                    after.attribute.set_foreground_color(AttributeColor::Palette(fg & 0x0f));
                    after.attribute.set_is_bold(false);
                }
                if let Some(bg) = edit.bg {
                    let bg = if ice { bg & 0x0f } else { bg & 0x07 };
                    after.attribute.set_background_color(AttributeColor::Palette(bg));
                    after.attribute.set_is_blinking(false);
                }
                if !after.is_visible() {
                    // A cell icy_engine never filled: give it a real attribute.
                    after.attribute = TextAttribute::new(edit.fg.unwrap_or(7) as u32, edit.bg.unwrap_or(0) as u32);
                }
                after
            })
        }))
    }

    /// Changes cells as part of stroke `stroke`, each to what its function
    /// makes of the cell as it is (so a later change to the same cell sees
    /// the earlier one). The one path every edit takes into the undo history.
    pub(crate) fn commit<F: FnOnce(AttributedChar) -> AttributedChar>(&mut self, stroke: u32, cells: impl IntoIterator<Item = (i32, i32, F)>) -> Option<CellRect> {
        let layer_index = self.edit_layer();
        let columns = self.buffer.width();
        let mut changed: Vec<(i32, i32, AttributedChar, AttributedChar)> = Vec::new();
        let mut index: HashMap<(i32, i32), usize> = HashMap::new();
        let mut dirty: Option<CellRect> = None;
        for (x, y, change) in cells {
            if x < 0 || y < 0 || x >= columns || y >= self.rows {
                continue;
            }
            let layer = &mut self.buffer.layers[layer_index];
            let before = layer.char_at((x, y).into());
            let after = change(before);
            if after == before {
                continue;
            }
            layer.set_char((x, y), after);
            // An edited cell isn't one the iCE switch moved any more.
            self.switched.remove(&(layer_index, x, y));
            // A cell changed twice keeps its first "before", so undo
            // restores it whatever order the changes came in.
            match index.get(&(x, y)) {
                Some(&i) => changed[i].3 = after,
                None => {
                    index.insert((x, y), changed.len());
                    changed.push((x, y, before, after));
                }
            }
            let cell = CellRect::cell(x, y);
            dirty = Some(dirty.map_or(cell, |d| d.union(cell)));
        }
        if changed.is_empty() {
            return None;
        }
        self.buffer.mark_dirty();
        let saved_now = self.history.revision() == self.history.saved;
        match self.history.undo.last_mut() {
            Some((_, Step::Cells { stroke: s, layer, cells }))
                if *s == stroke && *layer == layer_index && self.history.redo.is_empty() && !saved_now =>
            {
                // Same stroke: keep each cell's first "before".
                for (x, y, before, after) in changed {
                    if let Some(existing) = cells.iter_mut().find(|c| c.0 == x && c.1 == y) {
                        existing.3 = after;
                    } else {
                        cells.push((x, y, before, after));
                    }
                }
            }
            _ => self.push_step(Step::Cells { stroke, layer: layer_index, cells: changed }),
        }
        dirty
    }

    /// Takes back stroke `stroke` if it's the last step, without keeping it
    /// for redo: a shape being dragged is redrawn from scratch at each move.
    /// Returns the cells it had changed.
    pub(crate) fn retract(&mut self, stroke: u32) -> Option<CellRect> {
        if !self.history.redo.is_empty() {
            return None;
        }
        match self.history.undo.last() {
            Some((_, Step::Cells { stroke: s, .. })) if *s == stroke => {}
            _ => return None,
        }
        let (_, step) = self.history.undo.pop()?;
        Some(self.restore(&step, true))
    }

    fn push_step(&mut self, step: Step) {
        self.history.last += 1;
        self.history.undo.push((self.history.last, step));
        self.history.redo.clear();
    }

    /// Resizes the canvas, keeping the art at the top left. New cells are
    /// blank. Undoable.
    pub fn resize(&mut self, columns: i32, rows: i32) -> Result<(), String> {
        if !(1..=MAX_COLUMNS).contains(&columns) || !(1..=MAX_ROWS).contains(&rows) {
            return Err(format!("A canvas can be 1 to {MAX_COLUMNS} columns and 1 to {MAX_ROWS} rows"));
        }
        if columns == self.buffer.width() && rows == self.rows {
            return Ok(());
        }
        let before = (self.buffer.size(), self.rows, self.buffer.layers.clone());
        let blank = AttributedChar::new(' ', TextAttribute::new(7, 0));
        let old_width = self.buffer.width();
        let old_rows = self.rows;
        self.buffer.set_size((columns, rows));
        for layer in &mut self.buffer.layers {
            layer.set_size((columns, rows));
        }
        let base = &mut self.buffer.layers[0];
        for y in 0..rows {
            for x in 0..columns {
                if (x >= old_width || y >= old_rows) || !base.char_at((x, y).into()).is_visible() {
                    base.set_char((x, y), blank);
                }
            }
        }
        self.rows = rows;
        self.switched.retain(|&(_, x, y)| x < columns && y < rows);
        self.buffer.mark_dirty();
        let after = (self.buffer.size(), self.rows, self.buffer.layers.clone());
        self.push_step(Step::Resize { before, after });
        Ok(())
    }

    /// Takes back the last step. Returns the cells to redraw (the whole
    /// canvas after a resize), or `None` when there was nothing to undo.
    pub fn undo(&mut self) -> Option<CellRect> {
        let (revision, step) = self.history.undo.pop()?;
        let rect = self.restore(&step, true);
        self.history.redo.push((revision, step));
        Some(rect)
    }

    pub fn redo(&mut self) -> Option<CellRect> {
        let (revision, step) = self.history.redo.pop()?;
        let rect = self.restore(&step, false);
        self.history.undo.push((revision, step));
        Some(rect)
    }

    fn restore(&mut self, step: &Step, undo: bool) -> CellRect {
        self.buffer.mark_dirty();
        match step {
            Step::Cells { layer, cells, .. } => {
                let mut rect: Option<CellRect> = None;
                for &(x, y, before, after) in cells {
                    // Each cell appears once, so the order doesn't matter.
                    self.buffer.layers[*layer].set_char((x, y), if undo { before } else { after });
                    self.switched.remove(&(*layer, x, y));
                    let cell = CellRect::cell(x, y);
                    rect = Some(rect.map_or(cell, |r| r.union(cell)));
                }
                rect.unwrap_or(CellRect::cell(0, 0))
            }
            Step::Resize { before, after } => {
                let (size, rows, layers) = if undo { before } else { after };
                self.buffer.set_size(*size);
                self.buffer.layers = layers.clone();
                self.rows = *rows;
                self.switched.retain(|&(_, x, y)| x < size.width && y < *rows);
                CellRect { x: 0, y: 0, width: size.width, height: *rows }
            }
        }
    }

    /// What saving in `extension`'s format would lose. Empty when nothing.
    pub fn save_losses(&self, extension: &str) -> Result<Vec<SaveLoss>, String> {
        let format = save_format(extension)?;
        let buffer = self.buffer_for_save();
        let mut losses = Vec::new();
        for issue in format.check_compatibility(&buffer) {
            // icy_engine flags iCE whenever the document is in iCE mode;
            // only cells with a bright background actually lose anything.
            if matches!(issue.issue_type, IssueType::IceColorsUnsupported) && !self.uses_bright_backgrounds() {
                continue;
            }
            let blocking = issue.severity == icy_engine::formats::IssueSeverity::Error;
            let message = match issue.issue_type {
                IssueType::OddWidthNotAllowed { width } => format!("{} needs an even number of columns; this is {width} wide", format.name()),
                IssueType::WidthExceeded { width, max } => format!("{} holds at most {max} columns; this is {width} wide", format.name()),
                IssueType::HeightExceeded { height, max } => format!("{} holds at most {max} rows; this is {height} tall", format.name()),
                IssueType::TruecolorUnsupported => "24-bit colors become the nearest of the 16 colors".to_string(),
                IssueType::CustomPaletteUnsupported => "The custom palette is replaced by the standard 16 colors".to_string(),
                IssueType::IceColorsUnsupported => "Bright backgrounds (iCE colors) become their dark versions".to_string(),
                IssueType::CustomFontUnsupported => "The custom font isn't saved; the file will show in IBM VGA".to_string(),
                IssueType::MultipleFontsUnsupported { font_count } => format!("Only one font is kept; this uses {font_count}"),
                _ => issue.message.clone(),
            };
            losses.push(SaveLoss { blocking, message });
        }
        let outside = self.characters_outside_font();
        if outside > 0 {
            losses.push(SaveLoss {
                blocking: true,
                message: format!("{outside} characters aren't in the font's 256, so {} can't hold them", format.name()),
            });
        }
        let requirements = buffer.analyze_capability_requirements();
        // Stylus writes ANSI for DOS-era viewers (16 colors), whatever
        // icy_engine's ANSI can hold, so 24-bit is lost there too.
        if format == FileFormat::Ansi && requirements.uses_truecolor {
            losses.push(SaveLoss { blocking: false, message: "24-bit colors become the nearest of the 16 colors".to_string() });
        }
        // Artworx is always 80 columns.
        if format == FileFormat::Artworx && buffer.width() != 80 {
            losses.push(SaveLoss { blocking: true, message: format!("Artworx is always 80 columns; this is {} wide", buffer.width()) });
        }
        if ice_only(format) && self.has_blink() {
            losses.push(SaveLoss { blocking: false, message: "Blinking text stops blinking".to_string() });
        }
        if format == FileFormat::Ascii && self.has_color() {
            losses.push(SaveLoss { blocking: false, message: "Plain text keeps no colors: only the characters are saved".to_string() });
        }
        if !format.capabilities().contains(FormatCapabilities::ICE_COLORS) && format != FileFormat::Ascii && self.has_blink() {
            losses.push(SaveLoss { blocking: false, message: "Blinking text stops blinking".to_string() });
        }
        Ok(losses)
    }

    /// Cells whose character has no code in the 256-character font (from a
    /// UTF-8 file, with no CP437 equivalent).
    fn characters_outside_font(&self) -> usize {
        self.buffer
            .layers
            .iter()
            .map(|layer| (0..self.rows).map(|y| (0..self.buffer.width()).filter(|&x| layer.char_at((x, y).into()).ch as u32 > 0xff).count()).sum::<usize>())
            .sum()
    }

    fn uses_bright_backgrounds(&self) -> bool {
        self.ice
            && self.buffer.layers.iter().any(|layer| {
                (0..self.rows).any(|y| {
                    (0..self.buffer.width()).any(|x| matches!(layer.char_at((x, y).into()).attribute.background_color(), AttributeColor::Palette(8..=15)))
                })
            })
    }

    fn has_color(&self) -> bool {
        let layer = &self.buffer.layers[self.edit_layer()];
        (0..self.rows).any(|y| {
            (0..self.buffer.width()).any(|x| {
                let a = layer.char_at((x, y).into()).attribute;
                !(matches!(a.foreground_color(), AttributeColor::Palette(7)) && matches!(a.background_color(), AttributeColor::Palette(0)))
                    || a.is_blinking()
                    || a.is_bold()
            })
        })
    }

    /// The document as a file in `extension`'s format, with one SAUCE
    /// record from `sauce` and the document's own size, flags and font.
    /// Writes nothing; the caller saves the bytes and calls `mark_saved`.
    pub fn save(&self, extension: &str, sauce: &SauceFields) -> Result<Vec<u8>, String> {
        let format = save_format(extension)?;
        if let Some(loss) = self.save_losses(extension)?.into_iter().find(|l| l.blocking) {
            return Err(loss.message);
        }
        let mut buffer = self.buffer_for_save();
        if ice_only(format) {
            // Blink (iCE off) is dropped, as save_losses says.
            buffer.ice_mode = IceMode::Ice;
        }
        let mut options = match format {
            FileFormat::Ansi => SaveOptions::ansi(AnsiCompatibilityLevel::Vt100),
            _ => SaveOptions::new(),
        };
        if let FormatOptions::Ansi(ansi) = &mut options.format {
            // Rows fill the SAUCE width, no line breaks, as scene art does.
            ansi.line_ending = icy_engine::formats::LineEnding::CrLf;
        }
        // Every cell as it is. icy_engine's optimizer would rewrite cells
        // that look the same (a black full block on red as a blank), and a
        // reopened ANSI then ends at an earlier row.
        options.preprocess.optimize_colors = false;
        options.preprocess.normalize_whitespaces = false;
        options.sauce = Some(SauceMetaData {
            title: sauce.title.as_str().into(),
            author: sauce.author.as_str().into(),
            group: sauce.group.as_str().into(),
            comments: sauce.comments.iter().map(|c| c.as_str().into()).collect(),
        });
        let mut bytes = format.to_bytes(&buffer, &options).map_err(|e| e.to_string())?;
        set_sauce_date(&mut bytes, &sauce.date);
        Ok(bytes)
    }

    /// The buffer cut to the rows shown (parsed formats pad to a screen).
    pub(crate) fn buffer_for_save(&self) -> icy_engine::TextBuffer {
        let mut buffer = self.buffer.clone();
        let size = Size::new(buffer.width(), self.rows);
        buffer.set_size(size);
        for layer in &mut buffer.layers {
            layer.set_size(size);
        }
        buffer
    }

    /// Shows a text file in another font (the Amiga's for Amiga ASCII).
    /// Only for files without a font of their own: changes how the art
    /// looks, not its characters.
    pub fn set_text_font(&mut self, name: &str) -> Result<(), String> {
        if !TEXT_FONTS.contains(&name) {
            return Err(format!("Stylus doesn't have the font {name}"));
        }
        let font = icy_engine::BitFont::from_sauce_name(name).map_err(|e| e.to_string())?;
        self.buffer.set_font(0, font);
        self.buffer.mark_dirty();
        Ok(())
    }
}

fn save_format(extension: &str) -> Result<FileFormat, String> {
    let extension = extension.to_ascii_lowercase();
    if !SAVE_FORMATS.iter().any(|f| f.extension == extension) {
        return Err(format!("Stylus can't save .{extension} files"));
    }
    FileFormat::from_extension(&extension).ok_or_else(|| format!("Stylus can't save .{extension} files"))
}

/// Formats that always have iCE colors (no blink): icy_engine's writers
/// refuse a document in blink mode.
fn ice_only(format: FileFormat) -> bool {
    matches!(format, FileFormat::Artworx | FileFormat::IceDraw)
}

fn palette_index(color: AttributeColor) -> u8 {
    match color {
        AttributeColor::Palette(n) | AttributeColor::ExtendedPalette(n) => n,
        AttributeColor::Rgb(r, g, b) => icy_engine::nearest_dos_color((r, g, b)) as u8,
        AttributeColor::Transparent => 0,
    }
}

/// Writes `date` (CCYYMMDD) into the SAUCE record at the end of `bytes`.
/// icy_engine's writer leaves the date empty.
fn set_sauce_date(bytes: &mut [u8], date: &str) {
    if date.len() != 8 || !date.bytes().all(|b| b.is_ascii_digit()) || bytes.len() < 128 {
        return;
    }
    let start = bytes.len() - 128;
    if &bytes[start..start + 7] != b"SAUCE00" {
        return;
    }
    let at = start + 7 + 35 + 20 + 20;
    bytes[at..at + 8].copy_from_slice(date.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderSettings;

    fn edit(x: i32, y: i32, code: u8, fg: u8, bg: u8) -> CellEdit {
        CellEdit { x, y, code: Some(code), fg: Some(fg), bg: Some(bg) }
    }

    fn sauce() -> SauceFields {
        SauceFields {
            title: "Stylus test".into(),
            author: "ansiapps".into(),
            group: "".into(),
            date: "20260930".into(),
            comments: vec!["made by a test".into()],
        }
    }

    /// A small colored piece: every foreground on every background it can have.
    fn sample(ice: bool) -> Document {
        let mut doc = Document::new_blank(80, 8, ice).unwrap();
        let bgs = if ice { 16 } else { 8 };
        let edits: Vec<CellEdit> = (0..16 * 8).map(|i| edit(i % 16, i / 16, [b'A', 0xdb, 0xb1, b'x'][(i % 4) as usize], (i % 16) as u8, ((i / 16) % bgs) as u8)).collect();
        doc.apply(1, &edits);
        doc
    }

    #[test]
    fn a_new_document_is_blank_and_unedited() {
        let doc = Document::new_blank(80, 25, true).unwrap();
        let info = doc.info();
        assert_eq!((info.columns, info.rows), (80, 25));
        assert!(!info.edited && !info.can_undo && !info.can_redo);
        assert_eq!(doc.cell(79, 24), Some(CellInfo { code: 32, fg: 7, bg: 0, blink: false, truecolor: false }));
        assert!(doc.cell(80, 0).is_none());
        assert!(Document::new_blank(0, 25, true).is_err());
    }

    #[test]
    fn edits_undo_and_redo_by_stroke() {
        let mut doc = Document::new_blank(10, 5, true).unwrap();
        // One stroke in two batches, then a second stroke.
        assert_eq!(doc.apply(1, &[edit(0, 0, b'a', 15, 1)]), Some(CellRect { x: 0, y: 0, width: 1, height: 1 }));
        doc.apply(1, &[edit(3, 2, b'b', 14, 4), edit(0, 0, b'c', 13, 2)]);
        doc.apply(2, &[edit(5, 4, b'd', 12, 3)]);
        assert!(doc.is_edited());
        assert_eq!(doc.cell(0, 0).unwrap().code, u32::from(b'c'));

        assert_eq!(doc.undo(), Some(CellRect { x: 5, y: 4, width: 1, height: 1 }));
        assert_eq!(doc.cell(5, 4).unwrap().code, 32);
        assert_eq!(doc.undo(), Some(CellRect { x: 0, y: 0, width: 4, height: 3 }));
        assert_eq!(doc.cell(0, 0), Some(CellInfo { code: 32, fg: 7, bg: 0, blink: false, truecolor: false }));
        assert!(!doc.is_edited() && !doc.can_undo());
        assert_eq!(doc.undo(), None);

        doc.redo();
        assert_eq!(doc.cell(0, 0).unwrap().code, u32::from(b'c'));
        assert_eq!(doc.cell(3, 2).unwrap().bg, 4);
        doc.redo();
        assert_eq!(doc.cell(5, 4).unwrap().code, u32::from(b'd'));
        assert!(!doc.can_redo());
    }

    #[test]
    fn a_new_stroke_after_undo_drops_the_redo() {
        let mut doc = Document::new_blank(10, 5, true).unwrap();
        doc.apply(1, &[edit(0, 0, b'a', 15, 1)]);
        doc.undo();
        doc.apply(2, &[edit(1, 0, b'b', 15, 1)]);
        assert!(!doc.can_redo());
        doc.undo();
        assert!(!doc.can_undo());
        assert_eq!(doc.cell(0, 0).unwrap().code, 32);
    }

    #[test]
    fn paint_color_only_or_character_only() {
        let mut doc = Document::new_blank(4, 1, true).unwrap();
        doc.apply(1, &[edit(0, 0, b'Q', 10, 5)]);
        doc.apply(2, &[CellEdit { x: 0, y: 0, code: None, fg: Some(12), bg: None }]);
        assert_eq!(doc.cell(0, 0), Some(CellInfo { code: u32::from(b'Q'), fg: 12, bg: 5, blink: false, truecolor: false }));
        doc.apply(3, &[CellEdit { x: 0, y: 0, code: Some(b'R'), fg: None, bg: None }]);
        assert_eq!(doc.cell(0, 0), Some(CellInfo { code: u32::from(b'R'), fg: 12, bg: 5, blink: false, truecolor: false }));
        // Painting what's already there changes nothing and adds no step.
        assert_eq!(doc.apply(4, &[CellEdit { x: 0, y: 0, code: Some(b'R'), fg: None, bg: None }]), None);
        doc.undo();
        assert_eq!(doc.cell(0, 0).unwrap().code, u32::from(b'Q'));
    }

    #[test]
    fn without_ice_backgrounds_are_the_eight_dark_colors() {
        let mut doc = Document::new_blank(4, 1, false).unwrap();
        doc.apply(1, &[edit(0, 0, b'x', 15, 9)]);
        assert_eq!(doc.cell(0, 0).unwrap().bg, 1);
    }

    #[test]
    fn saving_marks_the_document_unedited_until_the_next_change() {
        let mut doc = Document::new_blank(4, 1, true).unwrap();
        doc.apply(1, &[edit(0, 0, b'x', 15, 1)]);
        doc.mark_saved();
        assert!(!doc.is_edited());
        // The same stroke going on after the save is a change again.
        doc.apply(1, &[edit(1, 0, b'y', 15, 1)]);
        assert!(doc.is_edited());
        doc.undo();
        assert!(!doc.is_edited(), "undoing back to the save");
        doc.undo();
        assert!(doc.is_edited(), "undoing past the save");
    }

    #[test]
    fn resizing_keeps_the_art_and_undoes() {
        let mut doc = sample(true);
        let before = doc.render_rows(0, 8, true).rgba;
        doc.resize(90, 12).unwrap();
        assert_eq!((doc.info().columns, doc.info().rows), (90, 12));
        assert_eq!(doc.cell(89, 11), Some(CellInfo { code: 32, fg: 7, bg: 0, blink: false, truecolor: false }));
        assert_eq!(doc.cell(1, 0).unwrap().code, 0xdb);
        doc.resize(8, 4).unwrap();
        assert!(doc.cell(8, 0).is_none());
        doc.undo();
        doc.undo();
        assert_eq!((doc.info().columns, doc.info().rows), (80, 8));
        assert_eq!(doc.render_rows(0, 8, true).rgba, before);
        doc.redo();
        assert_eq!((doc.info().columns, doc.info().rows), (90, 12));
        assert!(doc.resize(0, 5).is_err());
    }

    #[test]
    fn render_cells_draws_just_the_rectangle() {
        let doc = sample(true);
        let band = doc.render_cells(CellRect { x: 2, y: 1, width: 3, height: 2 }, true);
        assert_eq!((band.width, band.height), (3 * 8, 2 * 16));
        // The same pixels as that part of the whole render.
        let whole = doc.render_rows(0, 8, true);
        for row in 0..band.height as usize {
            let src = ((16 + row) * whole.width as usize + 16) * 4;
            let dst = row * band.width as usize * 4;
            assert_eq!(&band.rgba[dst..dst + 24 * 4], &whole.rgba[src..src + 24 * 4]);
        }
    }

    #[test]
    fn every_save_format_reopens_as_the_same_art() {
        for ice in [true, false] {
            let doc = sample(ice);
            let expected = doc.render_rows(0, 8, true).rgba;
            for format in SAVE_FORMATS.iter().filter(|f| f.extension != "asc") {
                let losses = doc.save_losses(format.extension).unwrap();
                assert!(losses.iter().all(|l| !l.blocking), "{}: {losses:?}", format.extension);
                let bytes = doc.save(format.extension, &sauce()).unwrap();
                let reopened = Document::open(format.extension, &bytes).unwrap();
                let info = reopened.info();
                assert_eq!((info.columns, info.rows), (80, 8), "{} ice={ice}", format.extension);
                let mut reopened = reopened;
                reopened.set_settings(RenderSettings { ice_colors: ice, ..reopened.settings() });
                assert_eq!(reopened.render_rows(0, 8, true).rgba, expected, "{} ice={ice}", format.extension);
            }
        }
    }

    #[test]
    fn a_save_writes_exactly_one_sauce_record_with_the_fields() {
        let doc = sample(true);
        let bytes = doc.save("ans", &sauce()).unwrap();
        assert_eq!(bytes.windows(7).filter(|w| w == b"SAUCE00").count(), 1);
        // Saving what was opened from a file with SAUCE still writes one.
        let reopened = Document::open("ans", &bytes).unwrap();
        let again = reopened.save("ans", &sauce()).unwrap();
        assert_eq!(again.windows(7).filter(|w| w == b"SAUCE00").count(), 1);
        let info = Document::open("ans", &again).unwrap().info();
        let record = info.sauce.unwrap();
        assert_eq!(record.title, "Stylus test");
        assert_eq!(record.author, "ansiapps");
        assert_eq!(record.date, "2026/09/30");
        assert_eq!(record.comments, vec!["made by a test".to_string()]);
        assert_eq!(record.columns, Some(80));
        assert_eq!(record.ice_colors, Some(true));
    }

    #[test]
    fn plain_text_saves_the_characters_and_says_the_colors_go() {
        let mut doc = Document::new_blank(5, 2, true).unwrap();
        doc.apply(1, &[edit(0, 0, b'H', 7, 0), edit(1, 0, b'i', 7, 0), edit(0, 1, b'!', 7, 0)]);
        assert!(doc.save_losses("asc").unwrap().is_empty());
        let bytes = doc.save("asc", &sauce()).unwrap();
        assert!(bytes.starts_with(b"Hi"));
        doc.apply(2, &[edit(4, 1, b'x', 12, 1)]);
        let losses = doc.save_losses("asc").unwrap();
        assert_eq!(losses.len(), 1);
        assert!(losses[0].message.contains("no colors"));
    }

    #[test]
    fn a_save_that_cant_hold_the_art_says_why() {
        let doc = Document::new_blank(15, 2, true).unwrap();
        let losses = doc.save_losses("bin").unwrap();
        assert!(losses.iter().any(|l| l.blocking && l.message.contains("even number of columns")));
        assert!(doc.save("bin", &sauce()).is_err());
        assert!(doc.save("png", &sauce()).is_err());
        assert!(doc.save_losses("ans").unwrap().is_empty());
        assert!(doc.save_losses("adf").unwrap().iter().any(|l| l.blocking && l.message.contains("80 columns")));
    }

    #[test]
    fn blinking_text_is_named_when_the_format_cant_blink() {
        // A BIN with blink on (iCE off): TundraDraw has no blink.
        let data: Vec<u8> = (0..160).flat_map(|_| [b'#', 0x9f]).collect();
        let mut doc = Document::open("bin", &data).unwrap();
        assert!(doc.save_losses("ans").unwrap().is_empty());
        let losses = doc.save_losses("asc").unwrap();
        assert!(losses.iter().any(|l| l.message.contains("no colors")));
        doc.resize(80, 1).unwrap();
        assert!(doc.save_losses("adf").unwrap().iter().any(|l| !l.blocking && l.message.contains("stops blinking")));
    }

    #[test]
    fn text_files_can_be_shown_in_an_amiga_font() {
        let mut doc = Document::open("asc", b"Amiga ASCII \xa9").unwrap();
        let ibm = doc.render_rows(0, 1, true).rgba;
        doc.set_text_font("Amiga Topaz 1+").unwrap();
        assert!(doc.info().font.contains("Topaz"), "{}", doc.info().font);
        assert_ne!(doc.render_rows(0, 1, true).rgba, ibm);
        assert!(!doc.is_edited(), "a font to view in isn't an edit");
        doc.set_text_font("IBM VGA").unwrap();
        assert_eq!(doc.render_rows(0, 1, true).rgba, ibm);
        assert!(doc.set_text_font("Comic Sans").is_err());
    }

    /// icy_tools#188 (fixed upstream as `da0d287`): TundraDraw's writer
    /// started from the wrong attribute, so black text before the first
    /// color change came back light gray.
    #[test]
    fn tundra_draw_keeps_black_text_at_the_start() {
        let mut doc = Document::new_blank(80, 1, true).unwrap();
        doc.apply(1, &[edit(0, 0, b'A', 0, 1)]);
        let bytes = doc.save("tnd", &sauce()).unwrap();
        assert_eq!(Document::open("tnd", &bytes).unwrap().cell(0, 0).unwrap().fg, 0);
    }

    #[test]
    fn utf8_text_shows_its_block_characters() {
        // UTF-8 with a BOM: ▀ ▄ █ and one character CP437 hasn't (€).
        let data = "\u{feff}\u{2580}\u{2584}\u{2588}x".as_bytes();
        let mut doc = Document::open("txt", data).unwrap();
        assert_eq!((0..4).map(|x| doc.cell(x, 0).unwrap().code).collect::<Vec<_>>(), vec![0xdf, 0xdc, 0xdb, u32::from(b'x')]);
        assert!(doc.save_losses("ans").unwrap().is_empty());
        doc.apply(1, &[CellEdit { x: 3, y: 0, code: None, fg: Some(7), bg: None }]);
        let euro = Document::open("txt", "\u{feff}\u{20ac}".as_bytes()).unwrap();
        assert!(euro.save_losses("xb").unwrap().iter().any(|l| l.blocking && l.message.contains("aren't in the font")));
    }
}
