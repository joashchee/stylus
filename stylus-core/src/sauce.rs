//! SAUCE records as Stylus shows them: every field, decoded from CP437.
//!
//! SAUCE text fields are CP437 bytes padded with spaces or NULs, not UTF-8,
//! so a group name like "Fûel" needs the code page to come out right.

use icy_sauce::{AspectRatio, Capabilities, LetterSpacing, SauceRecord};
use serde::Serialize;

/// Unicode for CP437 bytes 0x80..=0xFF. Bytes below 0x80 are ASCII here:
/// in SAUCE fields, control codes are padding, not the glyphs art uses.
const CP437_HIGH: &str = "ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒáíóúñÑªº¿⌐¬½¼¡«»░▒▓│┤╡╢╖╕╣║╗╝╜╛┐└┴┬├─┼╞╟╚╔╩╦╠═╬╧╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀αßΓπΣσµτΦΘΩδ∞φε∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■\u{a0}";

/// Decodes a SAUCE text field: CP437, trailing spaces and NULs removed.
pub fn decode_cp437(bytes: &[u8]) -> String {
    let high: Vec<char> = CP437_HIGH.chars().collect();
    let end = bytes.iter().rposition(|&b| b != b' ' && b != 0).map_or(0, |i| i + 1);
    bytes[..end]
        .iter()
        .map(|&b| match b {
            0 => ' ',
            0x01..=0x7f => b as char,
            _ => high[(b - 0x80) as usize],
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SauceInfo {
    pub title: String,
    pub author: String,
    pub group: String,
    /// As icy_sauce formats it, "YYYY/MM/DD", or empty.
    pub date: String,
    pub comments: Vec<String>,
    /// SAUCE data type and file type, e.g. "Character / Ansi".
    pub kind: String,
    /// Size of the file without SAUCE, as recorded.
    pub file_size: u32,
    /// Only for character and binary text files.
    pub columns: Option<u16>,
    pub lines: Option<u16>,
    pub ice_colors: Option<bool>,
    pub letter_spacing: Option<String>,
    pub aspect_ratio: Option<String>,
    pub font: Option<String>,
}

impl SauceInfo {
    pub fn from_record(record: &SauceRecord) -> Self {
        let mut info = SauceInfo {
            title: decode_cp437(record.title()),
            author: decode_cp437(record.author()),
            group: decode_cp437(record.group()),
            date: record.date().to_string().trim().to_string(),
            comments: record.comments().iter().map(|c| decode_cp437(c)).collect(),
            kind: format!("{:?}", record.data_type()),
            file_size: record.file_size(),
            columns: None,
            lines: None,
            ice_colors: None,
            letter_spacing: None,
            aspect_ratio: None,
            font: None,
        };
        let (format, columns, lines, ice, spacing, aspect, font) = match record.capabilities() {
            Some(Capabilities::Character(c)) => (format!("{:?}", c.format), c.columns, c.lines, c.ice_colors, c.letter_spacing, c.aspect_ratio, c.font_opt),
            Some(Capabilities::Binary(b)) => (format!("{:?}", b.format), b.columns, b.lines, b.ice_colors, b.letter_spacing, b.aspect_ratio, b.font_opt),
            _ => return info,
        };
        info.kind = format!("{} / {}", info.kind, format);
        info.columns = Some(columns);
        info.lines = Some(lines);
        info.ice_colors = Some(ice);
        info.letter_spacing = Some(
            match spacing {
                LetterSpacing::Legacy => "Not set",
                LetterSpacing::EightPixel => "8 px",
                LetterSpacing::NinePixel => "9 px",
                _ => "Unknown",
            }
            .to_string(),
        );
        info.aspect_ratio = Some(
            match aspect {
                AspectRatio::Legacy => "Not set",
                AspectRatio::LegacyDevice => "Stretched (legacy device)",
                AspectRatio::Square => "Square pixels",
                _ => "Unknown",
            }
            .to_string(),
        );
        info.font = font.map(|f| decode_cp437(&f)).filter(|f| !f.is_empty());
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cp437_table_covers_the_high_half() {
        assert_eq!(CP437_HIGH.chars().count(), 128);
    }

    #[test]
    fn decodes_cp437_and_trims_padding() {
        assert_eq!(decode_cp437(b"F\x96el  \0\0"), "Fûel");
        assert_eq!(decode_cp437(b"\xdb\xb2\xb1\xb0"), "█▓▒░");
        assert_eq!(decode_cp437(b"    "), "");
    }
}
