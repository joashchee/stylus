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
//! "STYLUS-RECOVERY2"            magic, 16 bytes
//! settings                      u8: bit 0 9-px, bit 1 iCE, bit 2 aspect
//! format name                   u16 LE length, UTF-8
//! SAUCE record                  u32 LE length (0 for none), as written
//! frames, frame shown, layer    u16 LE each
//! each frame                    u32 LE hold (ms), u32 LE length, IcyDraw bytes
//! ```
//!
//! IcyDraw holds one picture, so each frame is one IcyDraw file, with every
//! layer. Version 1 (before frames) had no frame fields and one IcyDraw
//! file to the end; it still opens.

use icy_engine::formats::{FileFormat, FormatOptions, IcyDrawFormatOptions, SaveOptions};
use icy_engine::TextPane;
use icy_sauce::SauceRecord;

use crate::document::{Document, RenderSettings};
use crate::frames::{Frame, DEFAULT_HOLD_MS};

const MAGIC: &[u8; 16] = b"STYLUS-RECOVERY2";
const MAGIC_V1: &[u8; 16] = b"STYLUS-RECOVERY1";

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
        for n in [self.frames.len(), self.frame, self.layer] {
            out.extend((n as u16).to_le_bytes());
        }
        for (f, frame) in self.frames.iter().enumerate() {
            let art = FileFormat::IcyDraw.to_bytes(&self.buffer_for_frame(f), &options).map_err(|e| e.to_string())?;
            out.extend(frame.hold_ms.to_le_bytes());
            out.extend((art.len() as u32).to_le_bytes());
            out.extend(art);
        }
        Ok(out)
    }

    /// Reopens a snapshot as it was: the art, its SAUCE record, format name
    /// and render settings. It opens as edited (its changes were never
    /// saved) with no undo history.
    pub fn from_recovery(data: &[u8]) -> Result<Document, String> {
        let bad = || "That isn't a Stylus recovery file".to_string();
        let (v1, rest) = match (data.strip_prefix(MAGIC.as_slice()), data.strip_prefix(MAGIC_V1.as_slice())) {
            (Some(rest), _) => (false, rest),
            (None, Some(rest)) => (true, rest),
            (None, None) => return Err(bad()),
        };
        let (&flags, rest) = rest.split_first().ok_or_else(bad)?;
        let (format, rest) = take(rest, 2).ok_or_else(bad)?;
        let (format, rest) = take(rest, u16::from_le_bytes([format[0], format[1]]) as usize).ok_or_else(bad)?;
        let format = String::from_utf8(format.to_vec()).map_err(|_| bad())?;
        let (sauce_len, rest) = take(rest, 4).ok_or_else(bad)?;
        let (sauce, rest) = take(rest, u32::from_le_bytes(sauce_len.try_into().map_err(|_| bad())?) as usize).ok_or_else(bad)?;
        let sauce = if sauce.is_empty() { None } else { SauceRecord::from_bytes(sauce).map_err(|e| e.to_string())? };

        let damaged = |why: &str| format!("The recovery file is damaged: {why}");
        // Each frame's hold and art.
        let mut frames: Vec<(u32, &[u8])> = Vec::new();
        let (count, shown, layer) = if v1 {
            frames.push((DEFAULT_HOLD_MS, rest));
            (1, 0, None)
        } else {
            let (head, mut rest) = take(rest, 6).ok_or_else(bad)?;
            let n = |i: usize| u16::from_le_bytes([head[i], head[i + 1]]) as usize;
            for _ in 0..n(0) {
                let (hold, more) = take(rest, 4).ok_or_else(bad)?;
                let (len, more) = take(more, 4).ok_or_else(bad)?;
                let (art, more) = take(more, u32::from_le_bytes(len.try_into().map_err(|_| bad())?) as usize).ok_or_else(bad)?;
                frames.push((u32::from_le_bytes(hold.try_into().map_err(|_| bad())?), art));
                rest = more;
            }
            (n(0), n(2), Some(n(4)))
        };
        if count == 0 || shown >= count {
            return Err(damaged("its frames are missing"));
        }

        let mut document: Option<Document> = None;
        for (hold_ms, art) in frames {
            let loaded = FileFormat::IcyDraw.from_bytes(art, None).map_err(|e| damaged(&e.to_string()))?;
            let buffer = loaded.screen.buffer;
            if buffer.width() <= 0 || buffer.height() <= 0 {
                return Err(damaged("it has no art in it"));
            }
            match &mut document {
                None => {
                    let rows = buffer.height();
                    let ice = matches!(buffer.ice_mode, icy_engine::IceMode::Ice);
                    let mut first = Document::from_buffer(buffer, sauce.clone(), &format, rows, ice);
                    first.frames[0].hold_ms = hold_ms;
                    document = Some(first);
                }
                Some(doc) => {
                    if buffer.size() != doc.buffer.size() || buffer.layers.len() != doc.buffer.layers.len() {
                        return Err(damaged("its frames don't match"));
                    }
                    doc.frames.push(Frame { layers: buffer.layers, hold_ms });
                }
            }
        }
        let mut document = document.ok_or_else(|| damaged("its frames are missing"))?;
        document.show_frame(shown);
        if let Some(layer) = layer {
            document.select_layer(layer).map_err(|_| damaged("its current layer is missing"))?;
        }
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
    fn every_frame_and_layer_comes_back() {
        let mut doc = Document::new_blank(20, 3, true).unwrap();
        doc.apply(1, &[edit(0, 0, b'a', 15, 1)]);
        doc.add_layer("Top").unwrap();
        doc.apply(2, &[edit(1, 0, b'T', 14, 12)]);
        doc.insert_frame(1, true).unwrap();
        doc.apply(3, &[edit(2, 0, b'2', 13, 9)]);
        doc.set_frame_hold(1, 400).unwrap();
        doc.set_layer_visible(1, false).unwrap();
        doc.select_layer(0).unwrap();
        doc.set_settings(RenderSettings { ice_colors: false, ..doc.settings() });
        let recovered = Document::from_recovery(&doc.recovery_snapshot().unwrap()).unwrap();
        let (a, b) = (doc.info(), recovered.info());
        assert_eq!((b.frames, b.frame, b.layers, b.layer), (a.frames, a.frame, a.layers, a.layer));
        assert_eq!(recovered.frame_hold(1), Some(400));
        let mut recovered = recovered;
        for f in 0..2 {
            doc.select_frame(f).unwrap();
            recovered.select_frame(f).unwrap();
            assert_same_art(&doc, &recovered);
        }
        // Switching iCE back restores the cells in every frame.
        doc.set_layer_visible(1, true).unwrap();
        recovered.set_layer_visible(1, true).unwrap();
        for d in [&mut doc, &mut recovered] {
            d.set_settings(RenderSettings { ice_colors: true, ..d.settings() });
        }
        assert_same_art(&doc, &recovered);
    }

    #[test]
    fn a_version_1_snapshot_still_opens() {
        let mut doc = Document::new_blank(8, 2, true).unwrap();
        doc.apply(1, &[edit(0, 0, b'v', 15, 1)]);
        // Version 1: the same header without the frame fields, then one
        // IcyDraw file.
        let v2 = doc.recovery_snapshot().unwrap();
        let header = 16 + 1 + 2 + 4 + (v2[17] as usize | (v2[18] as usize) << 8);
        let header = header + u32::from_le_bytes(v2[header - 4..header].try_into().unwrap()) as usize;
        let mut v1 = b"STYLUS-RECOVERY1".to_vec();
        v1.extend(&v2[16..header]);
        v1.extend(&v2[header + 6 + 8..]);
        let recovered = Document::from_recovery(&v1).unwrap();
        assert_same_art(&doc, &recovered);
        assert_eq!(recovered.frame_count(), 1);
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
