//! Stylus's engine, on top of icy_tools. Tauri-free: the desktop app wraps
//! it in Tauri commands (src-tauri/src/lib.rs), and the web build will
//! compile it to WebAssembly. See Diskette's docs/stylus-notes.md,
//! "Architecture".

pub mod contrast;
pub mod convert;
mod document;
mod edit;
mod export;
mod frames;
mod recovery;
mod sauce;
mod tools;

pub use document::{Band, Document, DocumentInfo, RenderSettings};
pub use frames::{DEFAULT_HOLD_MS, MAX_FRAMES, MAX_HOLD_MS, MAX_LAYERS, MIN_HOLD_MS};
pub use export::{PngExport, PngOptions, PngSize};
pub use edit::{CellEdit, CellInfo, CellRect, SaveFormat, SaveLoss, SauceFields, MAX_COLUMNS, MAX_ROWS, SAVE_FORMATS, TEXT_FONTS};
pub use sauce::{decode_cp437, SauceInfo};
pub use tools::{line_points, rect_between, Clip, Pen, Point, SelectionOp, Shape};

use serde::Serialize;

/// The icy_tools git revision in Cargo.toml. Keep the two in step.
pub const ICY_TOOLS_REV: &str = "da0d287fca7cc4028ba3c029d12be2ea1675bf9a";

/// File extensions the viewer opens (build step 2's v1 formats, plus what
/// icy_engine loads besides). Lower case, without the dot. The file dialog
/// filter and drag-and-drop both use this list.
pub const OPEN_EXTENSIONS: &[&str] = &[
    "ans", "ice", "asc", "nfo", "diz", "txt", "bin", "xb", "adf", "idf", "tnd", "pcb", "avt", "icy", "msg", "an1", "an2", "an3", "an4", "an5", "an6", "an7",
    "an8", "an9", "pet", "seq", "ata", "xep", "xp",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreInfo {
    pub version: String,
    pub icy_tools_rev: String,
}

pub fn core_info() -> CoreInfo {
    CoreInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        icy_tools_rev: ICY_TOOLS_REV.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_ansi_without_sauce_at_80_columns() {
        let doc = Document::open("ans", b"\x1b[31mHI\x1b[0m").unwrap();
        let info = doc.info();
        assert_eq!(info.columns, 80);
        assert!(info.rows >= 1);
        assert!(info.sauce.is_none());
    }

    #[test]
    fn parsed_formats_end_at_the_last_row_with_content() {
        let info = Document::open("ans", b"one\r\ntwo\r\nthree").unwrap().info();
        assert_eq!(info.rows, 3);
        assert_eq!(info.pixel_height, 3 * info.cell_height);
    }

    #[test]
    fn ascii_files_with_ansi_codes_are_read_as_ansi() {
        let plain = Document::open("asc", b"hello").unwrap();
        assert_eq!(plain.info().format, "ASCII");
        let colored = Document::open("asc", b"\x1b[31mhello\x1b[0m").unwrap();
        assert_eq!(colored.info().format, "ANSI");
        // The escape codes aren't drawn: the same pixels as the plain text in red.
        let ansi = Document::open("ans", b"\x1b[31mhello\x1b[0m").unwrap();
        assert_eq!(colored.render_rows(0, 1, true).rgba, ansi.render_rows(0, 1, true).rgba);
    }

    #[test]
    fn binary_formats_keep_blank_rows() {
        // Two rows of 160 cells, the second blank (space on black).
        let data: Vec<u8> = (0..160).flat_map(|_| [b'#', 0x1f]).chain((0..160).flat_map(|_| [b' ', 0x00])).collect();
        assert_eq!(Document::open("bin", &data).unwrap().info().rows, 2);
    }

    #[test]
    fn opens_bin_without_sauce_at_160_columns() {
        // One row of 160 cells: char, attribute.
        let data: Vec<u8> = (0..160).flat_map(|_| [b'#', 0x1f]).collect();
        let info = Document::open("bin", &data).unwrap().info();
        assert_eq!(info.columns, 160);
    }

    #[test]
    fn reads_every_sauce_field() {
        let mut data = b"HI".to_vec();
        data.push(0x1a); // EOF before SAUCE
        // Flags: bits 1-2 = 10, 9-px letter spacing.
        data.extend_from_slice(&sauce_record("Stylus test", "art\u{0}ist", 0b0000_0100));
        let info = Document::open("ans", &data).unwrap().info();
        let sauce = info.sauce.unwrap();
        assert_eq!(sauce.title, "Stylus test");
        assert_eq!(sauce.author, "art ist");
        assert_eq!(sauce.date, "2026/09/29");
        assert_eq!(sauce.columns, Some(80));
        assert_eq!(sauce.letter_spacing.as_deref(), Some("9 px"));
        // SAUCE's 9-px flag reaches the render settings.
        assert!(info.settings.letter_spacing);
        assert_eq!(info.cell_width, 9);
    }

    #[test]
    fn switching_ice_colors_twice_gives_back_the_same_pixels() {
        // Blink on a blue background, then plain text.
        let mut doc = Document::open("ans", b"\x1b[5;44;37mBLINK\x1b[0m plain").unwrap();
        doc.set_settings(RenderSettings { ice_colors: false, ..doc.settings() });
        let blink = doc.render_rows(0, 1, true).rgba;
        assert!(doc.info().has_blink);

        doc.set_settings(RenderSettings { ice_colors: true, ..doc.settings() });
        assert!(!doc.info().has_blink);
        assert_ne!(doc.render_rows(0, 1, true).rgba, blink, "iCE shows a bright background");

        doc.set_settings(RenderSettings { ice_colors: false, ..doc.settings() });
        assert_eq!(doc.render_rows(0, 1, true).rgba, blink);
    }

    #[test]
    fn without_sauce_ice_starts_off_and_switching_it_on_shows_bright_backgrounds() {
        // A BIN without SAUCE: attribute 0x9f is blink + blue background.
        let data: Vec<u8> = (0..160).flat_map(|_| [b'#', 0x9f]).collect();
        let mut doc = Document::open("bin", &data).unwrap();
        assert!(!doc.settings().ice_colors, "the high bit was loaded as blink");
        assert!(doc.info().has_blink);
        let blink = doc.render_rows(0, 1, true).rgba;
        doc.set_settings(RenderSettings { ice_colors: true, ..doc.settings() });
        assert!(!doc.info().has_blink);
        assert_ne!(doc.render_rows(0, 1, true).rgba, blink);
        doc.set_settings(RenderSettings { ice_colors: false, ..doc.settings() });
        assert_eq!(doc.render_rows(0, 1, true).rgba, blink);
    }

    #[test]
    fn a_bright_background_and_blink_in_one_file_both_come_back() {
        // aixterm bright blue background (104), then blink on blue.
        let mut doc = Document::open("ans", b"\x1b[104;37mA\x1b[0;5;44;37mB").unwrap();
        let original = doc.render_rows(0, 1, true).rgba;
        let ice = !doc.settings().ice_colors;
        doc.set_settings(RenderSettings { ice_colors: ice, ..doc.settings() });
        doc.set_settings(RenderSettings { ice_colors: !ice, ..doc.settings() });
        assert_eq!(doc.render_rows(0, 1, true).rgba, original);
    }

    #[test]
    fn blink_off_hides_blinking_cells() {
        let mut doc = Document::open("ans", b"\x1b[5;37mX").unwrap();
        doc.set_settings(RenderSettings { ice_colors: false, ..doc.settings() });
        assert_ne!(doc.render_rows(0, 1, true).rgba, doc.render_rows(0, 1, false).rgba);
    }

    #[test]
    fn renders_bands_of_rows() {
        let doc = Document::open("ans", b"one\r\ntwo\r\nthree\r\nfour").unwrap();
        let info = doc.info();
        let band = doc.render_rows(1, 2, true);
        assert_eq!(band.width as i32, info.pixel_width);
        assert_eq!(band.height as i32, 2 * info.cell_height);
        assert_eq!(band.rgba.len(), (band.width * band.height * 4) as usize);
        // Past the end: clamped, not a panic.
        assert_eq!(doc.render_rows(info.rows, 5, true).height, 0);
    }

    #[test]
    fn aspect_correction_is_a_display_stretch() {
        let mut doc = Document::open("ans", b"X").unwrap();
        doc.set_settings(RenderSettings { aspect_ratio: false, letter_spacing: false, ..doc.settings() });
        assert_eq!(doc.info().aspect_stretch, 1.0);
        doc.set_settings(RenderSettings { aspect_ratio: true, ..doc.settings() });
        assert!((doc.info().aspect_stretch - 1.2).abs() < 1e-6);
        doc.set_settings(RenderSettings { letter_spacing: true, ..doc.settings() });
        assert!((doc.info().aspect_stretch - 1.35).abs() < 1e-6);
    }

    /// A 128-byte SAUCE 00 record: an 80-column ANSi (data type 1, file type 1).
    fn sauce_record(title: &str, author: &str, flags: u8) -> Vec<u8> {
        fn field(s: &str, len: usize) -> Vec<u8> {
            let mut v = s.as_bytes().to_vec();
            v.resize(len, b' ');
            v
        }
        let mut r = b"SAUCE00".to_vec();
        r.extend(field(title, 35));
        r.extend(field(author, 20));
        r.extend(field("", 20)); // group
        r.extend(field("20260929", 8));
        r.extend(2u32.to_le_bytes()); // file size
        r.push(1); // data type: character
        r.push(1); // file type: ANSi
        r.extend(80u16.to_le_bytes()); // tinfo1: width
        r.extend(1u16.to_le_bytes()); // tinfo2: height
        r.extend([0u8; 4]); // tinfo3, tinfo4
        r.push(0); // comments
        r.push(flags);
        r.extend([0u8; 22]); // font name
        assert_eq!(r.len(), 128);
        r
    }
}
