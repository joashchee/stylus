//! A piece of art opened for viewing and editing: icy_engine's buffer plus
//! the SAUCE record, and the render modes Stylus lets the user switch.
//! Editing is in `edit.rs`.
//!
//! The render modes (9-px letter spacing, iCE colors, aspect correction)
//! change how the art is shown, never the file. Rendering goes through
//! icy_engine's renderer, one band of rows at a time, so the caller can show
//! a determinate progress bar (rows rendered) and never hold one giant image.

use icy_engine::formats::FileFormat;
use icy_engine::{AttributeColor, BufferType, IceMode, Rectangle, RenderOptions, TextBuffer, TextPane};
use icy_sauce::SauceRecord;
use std::collections::HashSet;

use crate::edit::History;
use serde::{Deserialize, Serialize};

use crate::sauce::SauceInfo;

/// How the art is shown. Starts from the file's SAUCE (or icy_engine's
/// defaults for the format) and can be changed without touching the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderSettings {
    /// 9-px character cells (VGA text mode) instead of 8-px.
    pub letter_spacing: bool,
    /// iCE colors: the attribute's high bit is a bright background, not blink.
    pub ice_colors: bool,
    /// Stretch vertically to the original display's aspect. Applied when
    /// the art is displayed (`DocumentInfo::aspect_stretch`), not in the pixels.
    pub aspect_ratio: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentInfo {
    pub columns: i32,
    pub rows: i32,
    /// icy_engine's name for the format, e.g. "ANSI".
    pub format: String,
    pub font: String,
    pub cell_width: i32,
    pub cell_height: i32,
    pub pixel_width: i32,
    pub pixel_height: i32,
    /// Vertical stretch for aspect correction, 1.0 when it's off or the font
    /// needs none.
    pub aspect_stretch: f32,
    /// Some cell blinks. Only matters with iCE colors off.
    pub has_blink: bool,
    pub settings: RenderSettings,
    pub sauce: Option<SauceInfo>,
    pub can_undo: bool,
    pub can_redo: bool,
    /// Changed since it was opened or last saved.
    pub edited: bool,
}

/// One band of rendered rows, RGBA, `width * height * 4` bytes.
pub struct Band {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub struct Document {
    pub(crate) buffer: TextBuffer,
    pub(crate) sauce: Option<SauceRecord>,
    pub(crate) format: String,
    /// Rows shown. For parsed formats (ANSI, ASCII, PCBoard…) icy_engine
    /// starts from an 80×25 screen, so rows after the last one with content
    /// are padding and are left out, as libansilove does. Binary formats
    /// (BIN, XBin, ADF…) define their height, so every row counts.
    pub(crate) rows: i32,
    /// Whether icy_engine loaded the file with iCE colors (from SAUCE).
    /// Otherwise the attribute's high bit was stored as blink.
    loaded_ice: bool,
    /// iCE as currently shown.
    pub(crate) ice: bool,
    /// Cells changed to show the other iCE setting than the loaded one,
    /// as (layer, x, y), so switching back restores exactly those. An
    /// edited cell leaves the set.
    pub(crate) switched: HashSet<(usize, i32, i32)>,
    pub(crate) history: History,
}

impl Document {
    /// Opens art from its bytes, the format picked by file extension
    /// (without the dot). Never touches the file itself: callers read it.
    pub fn open(extension: &str, data: &[u8]) -> Result<Self, String> {
        let mut format = FileFormat::from_extension(&extension.to_ascii_lowercase())
            .ok_or_else(|| format!("Stylus doesn't know the .{extension} format"))?;
        // In the scene, .ASC, .NFO, .DIZ and .TXT mean "text art", and often
        // hold ANSI color codes anyway. icy_engine's ASCII loader would print
        // the escape codes as characters, so a file with ESC in it is read as
        // ANSI, as ansilove and scene viewers do.
        if format == FileFormat::Ascii && data.contains(&0x1b) {
            format = FileFormat::Ansi;
        }
        let loaded = format.from_bytes(data, None).map_err(|e| e.to_string())?;
        let mut buffer = loaded.screen.buffer;
        // A UTF-8 text or ANSI file is read as Unicode, but the IBM fonts
        // draw CP437 codes, so ▀ ▄ █ would come out blank. Show each
        // character as the font's own where CP437 has it.
        if buffer.buffer_type == BufferType::Unicode {
            for layer in &mut buffer.layers {
                for y in 0..layer.height() {
                    for x in 0..layer.width() {
                        let mut ch = layer.char_at((x, y).into());
                        if ch.ch as u32 > 0x7f {
                            if let Some(cp437) = BufferType::CP437.try_convert_from_unicode(ch.ch) {
                                ch.ch = cp437;
                                layer.set_char((x, y), ch);
                            }
                        }
                    }
                }
            }
            buffer.buffer_type = BufferType::CP437;
        }
        if buffer.width() <= 0 || buffer.height() <= 0 {
            return Err("The file has no art in it".to_string());
        }
        let rows = if format.uses_parser() {
            buffer.line_count().clamp(1, buffer.height())
        } else {
            buffer.height()
        };
        let loaded_ice = matches!(buffer.ice_mode, IceMode::Ice);
        Ok(Document::from_buffer(buffer, loaded.sauce_opt, format.name(), rows, loaded_ice))
    }

    pub(crate) fn from_buffer(buffer: TextBuffer, sauce: Option<SauceRecord>, format: &str, rows: i32, ice: bool) -> Self {
        Document {
            buffer,
            sauce,
            format: format.to_string(),
            rows,
            loaded_ice: ice,
            ice,
            switched: HashSet::new(),
            history: History::default(),
        }
    }

    pub fn columns(&self) -> i32 {
        self.buffer.width()
    }

    pub fn rows(&self) -> i32 {
        self.rows
    }

    pub fn settings(&self) -> RenderSettings {
        RenderSettings {
            letter_spacing: self.buffer.use_letter_spacing(),
            ice_colors: self.ice,
            aspect_ratio: self.buffer.use_aspect_ratio(),
        }
    }

    pub fn set_settings(&mut self, settings: RenderSettings) {
        self.buffer.set_use_letter_spacing(settings.letter_spacing);
        self.buffer.set_use_aspect_ratio(settings.aspect_ratio);
        if settings.ice_colors != self.ice {
            self.switch_ice_colors(settings.ice_colors);
        }
    }

    /// iCE is decided when icy_engine parses the file: the attribute byte's
    /// high bit becomes either a bright background (iCE, from SAUCE) or the
    /// blink flag (everything else, including files without SAUCE). In
    /// 16-color art those are the same bit, so showing the other setting
    /// moves it across on each cell that has it, and switching back undoes
    /// exactly those cells, so a file mixing real bright backgrounds with
    /// blink comes back unchanged. RGB and 256-color backgrounds have no such
    /// bit and are left alone.
    fn switch_ice_colors(&mut self, ice: bool) {
        if ice == self.loaded_ice {
            for (l, x, y) in std::mem::take(&mut self.switched) {
                let layer = &mut self.buffer.layers[l];
                let mut ch = layer.char_at((x, y).into());
                if let AttributeColor::Palette(bg) = ch.attribute.background_color() {
                    if ice {
                        ch.attribute.set_is_blinking(false);
                        ch.attribute.set_background_color(AttributeColor::Palette(bg + 8));
                    } else {
                        ch.attribute.set_is_blinking(true);
                        ch.attribute.set_background_color(AttributeColor::Palette(bg - 8));
                    }
                    layer.set_char((x, y), ch);
                }
            }
        } else {
            for (l, layer) in self.buffer.layers.iter_mut().enumerate() {
                for y in 0..layer.height() {
                    for x in 0..layer.width() {
                        let mut ch = layer.char_at((x, y).into());
                        let AttributeColor::Palette(bg) = ch.attribute.background_color() else {
                            continue;
                        };
                        if ice && bg < 8 && ch.attribute.is_blinking() {
                            ch.attribute.set_is_blinking(false);
                            ch.attribute.set_background_color(AttributeColor::Palette(bg + 8));
                        } else if !ice && (8..16).contains(&bg) {
                            ch.attribute.set_is_blinking(true);
                            ch.attribute.set_background_color(AttributeColor::Palette(bg - 8));
                        } else {
                            continue;
                        }
                        layer.set_char((x, y), ch);
                        self.switched.insert((l, x, y));
                    }
                }
            }
        }
        self.ice = ice;
        self.buffer.ice_mode = if ice { IceMode::Ice } else { IceMode::Blink };
    }

    pub(crate) fn has_blink(&self) -> bool {
        self.buffer.layers.iter().any(|layer| {
            (0..layer.height()).any(|y| (0..layer.width()).any(|x| layer.char_at((x, y).into()).attribute.is_blinking()))
        })
    }

    fn cell_size(&self) -> (i32, i32) {
        let (w, h) = self.buffer.font(0).map_or((8, 16), |f| (f.size().width, f.size().height));
        // icy_engine draws the 9th column at render time from an 8-px font.
        if self.buffer.use_letter_spacing() && w == 8 {
            (9, h)
        } else {
            (w, h)
        }
    }

    pub fn info(&self) -> DocumentInfo {
        let (cell_width, cell_height) = self.cell_size();
        let stretch = self.buffer.get_aspect_ratio_stretch_factor();
        DocumentInfo {
            columns: self.buffer.width(),
            rows: self.rows,
            format: self.format.clone(),
            font: self.buffer.font(0).map_or_else(String::new, |f| f.name().to_string()),
            cell_width,
            cell_height,
            pixel_width: self.buffer.width() * cell_width,
            pixel_height: self.rows * cell_height,
            aspect_stretch: if self.buffer.use_aspect_ratio() && stretch > 0.0 { stretch } else { 1.0 },
            has_blink: self.has_blink(),
            settings: self.settings(),
            sauce: self.sauce.as_ref().map(SauceInfo::from_record),
            can_undo: self.can_undo(),
            can_redo: self.can_redo(),
            edited: self.is_edited(),
        }
    }

    /// Renders a rectangle of cells (clamped to the art), for redrawing what
    /// an edit changed.
    pub fn render_cells(&self, rect: crate::edit::CellRect, blink_on: bool) -> Band {
        let x = rect.x.clamp(0, self.buffer.width());
        let y = rect.y.clamp(0, self.rows);
        let width = rect.width.clamp(0, self.buffer.width() - x);
        let height = rect.height.clamp(0, self.rows - y);
        if width == 0 || height == 0 {
            return Band { width: 0, height: 0, rgba: Vec::new() };
        }
        let options = RenderOptions {
            rect: Rectangle::from(x, y, width, height).into(),
            blink_on,
            ..Default::default()
        };
        let (size, rgba) = self.buffer.render_to_rgba_raw(&options, false);
        Band {
            width: size.width as u32,
            height: size.height as u32,
            rgba,
        }
    }

    /// Renders rows `first_row .. first_row + row_count` (clamped to the
    /// art). `blink_on` false hides blinking cells, for the blink animation.
    pub fn render_rows(&self, first_row: i32, row_count: i32, blink_on: bool) -> Band {
        let first = first_row.clamp(0, self.rows);
        let count = row_count.clamp(0, self.rows - first);
        if count == 0 {
            return Band { width: 0, height: 0, rgba: Vec::new() };
        }
        let options = RenderOptions {
            rect: Rectangle::from(0, first, self.buffer.width(), count).into(),
            blink_on,
            ..Default::default()
        };
        let (size, rgba) = self.buffer.render_to_rgba_raw(&options, false);
        Band {
            width: size.width as u32,
            height: size.height as u32,
            rgba,
        }
    }
}
