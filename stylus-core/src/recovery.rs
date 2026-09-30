//! Crash recovery: a snapshot of an open document that loses nothing, for
//! the desktop app's autosave to the app-data folder (docs/roadmap.md,
//! 1c "Files"). Never written over the user's file; a recovered document
//! opens as unsaved changes, so the user decides where it goes.
//!
//! The art is icy_engine's own IcyDraw format, which holds everything the
//! buffer can (layers, fonts, palette, 24-bit colors, blink). IcyDraw keeps
//! the render settings only through a SAUCE record it builds itself, and
//! would drop the file's own record (its date, its flags), so the snapshot
//! carries those beside the art:
//!
//! ```text
//! "STYLUS-RECOVERY1"            magic, 16 bytes
//! settings                      u8: bit 0 9-px, bit 1 iCE, bit 2 aspect
//! format name                   u16 LE length, UTF-8
//! SAUCE record                  u32 LE length (0 for none), as written
//! the art                       IcyDraw bytes, to the end
//! ```

use icy_engine::formats::{FileFormat, FormatOptions, IcyDrawFormatOptions, SaveOptions};
use icy_engine::TextPane;
use icy_sauce::SauceRecord;

use crate::document::{Document, RenderSettings};

const MAGIC: &[u8; 16] = b"STYLUS-RECOVERY1";

impl Document {
    /// A number that changes whenever the art does, so an autosave can skip
    /// writing a document it already has. Render settings aren't in it.
    pub fn version(&self) -> u64 {
        self.buffer.version()
    }

    /// The document as it is now, for `from_recovery`.
    pub fn recovery_snapshot(&self) -> Result<Vec<u8>, String> {
        let settings = self.settings();
        let mut out = MAGIC.to_vec();
        out.push(u8::from(settings.letter_spacing) | u8::from(settings.ice_colors) << 1 | u8::from(settings.aspect_ratio) << 2);
        let format = self.format.as_bytes();
        out.extend((format.len() as u16).to_le_bytes());
        out.extend(format);
        let mut sauce = Vec::new();
        if let Some(record) = &self.sauce {
            record.write_without_eof(&mut sauce).map_err(|e| e.to_string())?;
        }
        out.extend((sauce.len() as u32).to_le_bytes());
        out.extend(sauce);
        let mut options = SaveOptions::icy_draw();
        // No preview image: an autosave is never shown as a file.
        options.format = FormatOptions::IcyDraw(IcyDrawFormatOptions { skip_thumbnail: true, compress: true });
        out.extend(FileFormat::IcyDraw.to_bytes(&self.buffer_for_save(), &options).map_err(|e| e.to_string())?);
        Ok(out)
    }

    /// Reopens a snapshot as it was: the art, its SAUCE record, format name
    /// and render settings. It opens as edited (its changes were never
    /// saved) with no undo history.
    pub fn from_recovery(data: &[u8]) -> Result<Document, String> {
        let bad = || "That isn't a Stylus recovery file".to_string();
        let rest = data.strip_prefix(MAGIC.as_slice()).ok_or_else(bad)?;
        let (&flags, rest) = rest.split_first().ok_or_else(bad)?;
        let (format, rest) = take(rest, 2).ok_or_else(bad)?;
        let (format, rest) = take(rest, u16::from_le_bytes([format[0], format[1]]) as usize).ok_or_else(bad)?;
        let format = String::from_utf8(format.to_vec()).map_err(|_| bad())?;
        let (sauce_len, rest) = take(rest, 4).ok_or_else(bad)?;
        let (sauce, art) = take(rest, u32::from_le_bytes(sauce_len.try_into().map_err(|_| bad())?) as usize).ok_or_else(bad)?;
        let sauce = if sauce.is_empty() { None } else { SauceRecord::from_bytes(sauce).map_err(|e| e.to_string())? };

        let loaded = FileFormat::IcyDraw.from_bytes(art, None).map_err(|e| format!("The recovery file is damaged: {e}"))?;
        let buffer = loaded.screen.buffer;
        if buffer.width() <= 0 || buffer.height() <= 0 {
            return Err("The recovery file is damaged: it has no art in it".to_string());
        }
        let rows = buffer.height();
        let ice = matches!(buffer.ice_mode, icy_engine::IceMode::Ice);
        let mut document = Document::from_buffer(buffer, sauce, &format, rows, ice);
        document.set_settings(RenderSettings { letter_spacing: flags & 1 != 0, ice_colors: flags & 2 != 0, aspect_ratio: flags & 4 != 0 });
        document.history.saved = -1;
        Ok(document)
    }
}

fn take(data: &[u8], n: usize) -> Option<(&[u8], &[u8])> {
    (data.len() >= n).then(|| data.split_at(n))
}

#[cfg(test)]
mod tests {
    use crate::{CellEdit, Document, RenderSettings};

    fn edit(x: i32, y: i32, code: u8, fg: u8, bg: u8) -> CellEdit {
        CellEdit { x, y, code: Some(code), fg: Some(fg), bg: Some(bg) }
    }

    fn assert_same_art(a: &Document, b: &Document) {
        let (ia, ib) = (a.info(), b.info());
        assert_eq!((ia.columns, ia.rows, &ia.format, &ia.font), (ib.columns, ib.rows, &ib.format, &ib.font));
        assert_eq!(ia.settings, ib.settings);
        for blink in [true, false] {
            assert_eq!(a.render_rows(0, ia.rows, blink).rgba, b.render_rows(0, ib.rows, blink).rgba);
        }
    }

    #[test]
    fn a_snapshot_comes_back_exactly_and_unsaved() {
        let mut doc = Document::new_blank(80, 6, true).unwrap();
        let edits: Vec<CellEdit> = (0..16 * 6).map(|i| edit(i % 16, i / 16, [b'A', 0xdb, 0xb1][(i % 3) as usize], (i % 16) as u8, ((i / 16) % 16) as u8)).collect();
        doc.apply(1, &edits);
        doc.set_settings(RenderSettings { letter_spacing: true, aspect_ratio: true, ice_colors: true });
        let recovered = Document::from_recovery(&doc.recovery_snapshot().unwrap()).unwrap();
        assert_same_art(&doc, &recovered);
        let info = recovered.info();
        assert!(info.edited && !info.can_undo && !info.can_redo);
    }

    #[test]
    fn blink_sauce_and_the_format_name_come_back() {
        // Blink on blue, iCE off, and a SAUCE record with a title and date.
        let mut data = b"\x1b[5;44;37mBLINK\x1b[0m plain".to_vec();
        data.push(0x1a);
        let mut sauce = b"SAUCE00".to_vec();
        for (s, len) in [("Recovered", 35), ("ansiapps", 20), ("", 20), ("19961231", 8)] {
            let mut f = s.as_bytes().to_vec();
            f.resize(len, b' ');
            sauce.extend(f);
        }
        sauce.extend(0u32.to_le_bytes());
        sauce.extend([1, 1]);
        sauce.extend(80u16.to_le_bytes());
        sauce.extend(1u16.to_le_bytes());
        sauce.extend([0u8; 6]);
        sauce.extend([0u8; 22]);
        data.extend(sauce);
        let mut doc = Document::open("ans", &data).unwrap();
        doc.set_settings(RenderSettings { ice_colors: false, ..doc.settings() });
        doc.apply(1, &[edit(10, 0, b'!', 14, 1)]);
        let recovered = Document::from_recovery(&doc.recovery_snapshot().unwrap()).unwrap();
        assert_same_art(&doc, &recovered);
        let info = recovered.info();
        assert!(info.has_blink);
        assert_eq!(info.format, "ANSI");
        let record = info.sauce.unwrap();
        assert_eq!((record.title.as_str(), record.author.as_str(), record.date.as_str()), ("Recovered", "ansiapps", "1996/12/31"));
    }

    #[test]
    fn a_text_file_keeps_its_amiga_font() {
        let mut doc = Document::open("asc", b"Amiga ASCII").unwrap();
        doc.set_text_font("Amiga Topaz 1+").unwrap();
        doc.apply(1, &[edit(0, 0, b'a', 7, 0)]);
        assert_same_art(&doc, &Document::from_recovery(&doc.recovery_snapshot().unwrap()).unwrap());
    }

    #[test]
    fn the_version_moves_with_edits() {
        let mut doc = Document::new_blank(4, 1, true).unwrap();
        let v = doc.version();
        doc.apply(1, &[edit(0, 0, b'x', 15, 1)]);
        assert_ne!(doc.version(), v);
    }

    #[test]
    fn anything_else_is_refused() {
        assert!(Document::from_recovery(b"").is_err());
        assert!(Document::from_recovery(b"\x1b[31mHI").is_err());
        let mut truncated = Document::new_blank(4, 1, true).unwrap().recovery_snapshot().unwrap();
        truncated.truncate(20);
        assert!(Document::from_recovery(&truncated).is_err());
    }
}
