//! The contrast checker from docs/ansiapps-color-contrast.md, as a lint
//! on any document (docs/roadmap.md, Phase 1c; theme mode enforces it in
//! Phase 2).
//!
//! Ratios use the WCAG 2.2 relative-luminance formula on the colors the
//! art actually shows (the document's palette, 24-bit colors as they are,
//! bold as bright), so on the 16 VGA colors they match the doc's ranked
//! list. **Text** needs 4.5:1 (the VGA font is 16 px with no bold, so the
//! 3:1 large-text allowance never applies); a **graphic** character's two
//! colors need 3:1.
//!
//! Until Phase 2's Role tool paints a role per cell, the role comes from
//! the character: letters, digits, punctuation and the accented and Greek
//! letters are text; the symbols, shades, blocks and line drawing are
//! graphics; blanks and the full block show one color and are exempt.
//! Checking a graphic against its neighbors, and decorative cells, wait
//! for the roles.

use icy_engine::{ansi_to_internal_palette_index, AttributeColor, TextPane, XTERM_256_PALETTE};
use serde::Serialize;
use std::collections::HashMap;

use crate::document::Document;

/// Text needs this much contrast (WCAG 1.4.3, AA).
pub const TEXT_RATIO: f64 = 4.5;
/// AAA (WCAG 1.4.6).
pub const AAA_RATIO: f64 = 7.0;
/// Icons, borders and meaningful graphics (WCAG 1.4.11).
pub const GRAPHIC_RATIO: f64 = 3.0;

/// The 16 VGA colors in palette order, with the names the contrast doc uses.
pub const VGA_COLORS: [((u8, u8, u8), &str); 16] = [
    ((0x00, 0x00, 0x00), "Black"),
    ((0x00, 0x00, 0xaa), "Blue"),
    ((0x00, 0xaa, 0x00), "Green"),
    ((0x00, 0xaa, 0xaa), "Cyan"),
    ((0xaa, 0x00, 0x00), "Red"),
    ((0xaa, 0x00, 0xaa), "Magenta"),
    ((0xaa, 0x55, 0x00), "Brown"),
    ((0xaa, 0xaa, 0xaa), "Light gray"),
    ((0x55, 0x55, 0x55), "Dark gray"),
    ((0x55, 0x55, 0xff), "Light blue"),
    ((0x55, 0xff, 0x55), "Light green"),
    ((0x55, 0xff, 0xff), "Light cyan"),
    ((0xff, 0x55, 0x55), "Light red"),
    ((0xff, 0x55, 0xff), "Light magenta"),
    ((0xff, 0xff, 0x55), "Yellow"),
    ((0xff, 0xff, 0xff), "White"),
];

/// What a cell is for, which sets the contrast it needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Text,
    Graphic,
}

/// Where a ratio lands on the doc's scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Level {
    /// 7:1 or more.
    Aaa,
    /// 4.5:1 or more: passes for text.
    Aa,
    /// 3:1 or more: icons, borders and graphics only, never words.
    Graphic,
    Fail,
}

pub fn level(ratio: f64) -> Level {
    if ratio >= AAA_RATIO {
        Level::Aaa
    } else if ratio >= TEXT_RATIO {
        Level::Aa
    } else if ratio >= GRAPHIC_RATIO {
        Level::Graphic
    } else {
        Level::Fail
    }
}

fn luminance((r, g, b): (u8, u8, u8)) -> f64 {
    let channel = |c: u8| {
        let c = f64::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

/// The WCAG contrast ratio of two colors, 1.0 to 21.0, the same either way round.
pub fn contrast_ratio(a: (u8, u8, u8), b: (u8, u8, u8)) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// The contrast doc's name for one of the 16 VGA colors, or None.
pub fn color_name(rgb: (u8, u8, u8)) -> Option<&'static str> {
    VGA_COLORS.iter().find(|(c, _)| *c == rgb).map(|(_, name)| *name)
}

/// The role a character has until cells carry their own (Phase 2), or
/// None when it shows a single color and needs no contrast.
pub fn role_of(code: u32) -> Option<Role> {
    match code {
        // NUL, space, the full block, and the non-breaking space.
        0 | 32 | 219 | 255 => None,
        // ☺ ♥ ♦ ♣ ♠ • ◘ ○ … arrows, ⌂, the shades, line drawing, the half
        // blocks, and ■.
        1..=31 | 127 | 176..=223 | 254 => Some(Role::Graphic),
        _ => Some(Role::Text),
    }
}

/// Cells of one role in one color pair that don't have the contrast they
/// need, for the Problems list.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContrastProblem {
    pub role: Role,
    pub fg: (u8, u8, u8),
    pub bg: (u8, u8, u8),
    /// The contrast doc's names, when the color is one of the 16.
    pub fg_name: Option<&'static str>,
    pub bg_name: Option<&'static str>,
    pub ratio: f64,
    /// What this role needs: 4.5 for text, 3 for graphics.
    pub needs: f64,
    /// The failing cells, as `y * columns + x`, top to bottom.
    pub cells: Vec<u32>,
}

impl Document {
    /// A cell's foreground and background as shown: the document's palette,
    /// bold as bright, 24-bit and 256-color as icy_engine draws them.
    fn shown_colors(&self, x: i32, y: i32) -> (u32, (u8, u8, u8), (u8, u8, u8)) {
        let ch = self.buffer.char_at((x, y).into());
        let attr = ch.attribute;
        let bright = attr.is_bold();
        let rgb = |color: AttributeColor, bold: bool| match color {
            AttributeColor::Palette(n) => {
                let n = if bold && n < 8 { n + 8 } else { n };
                self.buffer.palette.rgb(u32::from(n))
            }
            AttributeColor::ExtendedPalette(n) => {
                let index = ansi_to_internal_palette_index(u32::from(n));
                if (index as usize) < self.buffer.palette.len() {
                    self.buffer.palette.rgb(index)
                } else {
                    XTERM_256_PALETTE[n as usize].1.rgb()
                }
            }
            AttributeColor::Rgb(r, g, b) => (r, g, b),
            AttributeColor::Transparent => (0, 0, 0),
        };
        (ch.ch as u32, rgb(attr.foreground_color(), bright), rgb(attr.background_color(), false))
    }

    /// Every cell short of the contrast its role needs, grouped by role and
    /// color pair: text first, then the lowest ratio first.
    pub fn contrast_problems(&self) -> Vec<ContrastProblem> {
        let columns = self.buffer.width();
        let mut groups: HashMap<(Role, (u8, u8, u8), (u8, u8, u8)), Vec<u32>> = HashMap::new();
        for y in 0..self.rows {
            for x in 0..columns {
                let (code, fg, bg) = self.shown_colors(x, y);
                let Some(role) = role_of(code) else { continue };
                // A graphic in one color looks like a solid block; hidden
                // text is still text nobody can read.
                if role == Role::Graphic && fg == bg {
                    continue;
                }
                let needs = if role == Role::Text { TEXT_RATIO } else { GRAPHIC_RATIO };
                if contrast_ratio(fg, bg) < needs {
                    groups.entry((role, fg, bg)).or_default().push((y * columns + x) as u32);
                }
            }
        }
        let mut problems: Vec<ContrastProblem> = groups
            .into_iter()
            .map(|((role, fg, bg), cells)| ContrastProblem {
                role,
                fg,
                bg,
                fg_name: color_name(fg),
                bg_name: color_name(bg),
                ratio: contrast_ratio(fg, bg),
                needs: if role == Role::Text { TEXT_RATIO } else { GRAPHIC_RATIO },
                cells,
            })
            .collect();
        problems.sort_by(|a, b| a.role.cmp(&b.role).then(a.ratio.total_cmp(&b.ratio)).then(a.cells[0].cmp(&b.cells[0])));
        problems
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit::CellEdit;

    fn named(name: &str) -> (u8, u8, u8) {
        VGA_COLORS.iter().find(|(_, n)| *n == name).unwrap().0
    }

    fn ratio(a: &str, b: &str) -> f64 {
        contrast_ratio(named(a), named(b))
    }

    #[test]
    fn ratios_match_the_contrast_doc() {
        assert_eq!(format!("{:.2}", ratio("Black", "White")), "21.00");
        assert_eq!(format!("{:.2}", ratio("Yellow", "Brown")), "4.91");
        assert_eq!(format!("{:.2}", ratio("Brown", "Yellow")), "4.91");
        assert_eq!(format!("{:.2}", ratio("Light red", "Blue")), "4.23");
        assert_eq!(format!("{:.2}", ratio("Blue", "Cyan")), "4.64");
        assert_eq!(format!("{:.1}", ratio("Light magenta", "Brown")), "2.0");
        assert_eq!(level(ratio("Black", "Cyan")), Level::Aaa);
        assert_eq!(level(ratio("Dark gray", "Yellow")), Level::Aa);
        assert_eq!(level(ratio("Red", "Light gray")), Level::Graphic);
        assert_eq!(level(ratio("Blue", "Black")), Level::Fail);
    }

    #[test]
    fn thirty_two_pairs_pass_and_thirteen_are_graphics_only() {
        let mut text = 0;
        let mut graphic = 0;
        for a in 0..16 {
            for b in a + 1..16 {
                match level(contrast_ratio(VGA_COLORS[a].0, VGA_COLORS[b].0)) {
                    Level::Aaa | Level::Aa => text += 1,
                    Level::Graphic => graphic += 1,
                    Level::Fail => {}
                }
            }
        }
        assert_eq!((text, graphic), (32, 13));
    }

    #[test]
    fn roles_come_from_the_character() {
        assert_eq!(role_of(u32::from(b'A')), Some(Role::Text));
        assert_eq!(role_of(u32::from(b'.')), Some(Role::Text));
        assert_eq!(role_of(0x82), Some(Role::Text)); // é
        assert_eq!(role_of(0xb0), Some(Role::Graphic)); // ░
        assert_eq!(role_of(0xc4), Some(Role::Graphic)); // ─
        assert_eq!(role_of(0xdf), Some(Role::Graphic)); // ▀
        assert_eq!(role_of(3), Some(Role::Graphic)); // ♥
        assert_eq!(role_of(32), None);
        assert_eq!(role_of(0xdb), None); // █
    }

    fn put(doc: &mut Document, stroke: u32, x: i32, code: u8, fg: u8, bg: u8) {
        doc.apply(stroke, &[CellEdit { x, y: 0, code: Some(code), fg: Some(fg), bg: Some(bg) }]);
    }

    #[test]
    fn lists_failing_cells_by_pair_text_first() {
        let mut doc = Document::new_blank(10, 2, true).unwrap();
        assert!(doc.contrast_problems().is_empty(), "blanks need no contrast");
        put(&mut doc, 1, 0, b'O', 12, 1); // light red on blue, 4.23: fails text
        put(&mut doc, 2, 1, b'K', 12, 1);
        put(&mut doc, 3, 2, b'!', 15, 1); // white on blue: passes
        put(&mut doc, 4, 3, 0xb1, 12, 1); // ▒ light red on blue: fine for a graphic
        put(&mut doc, 5, 4, 0xc4, 1, 0); // ─ blue on black, 2.4: fails a graphic
        put(&mut doc, 6, 5, 0xdc, 6, 6); // ▄ brown on brown: looks solid
        put(&mut doc, 7, 6, b'x', 7, 7); // hidden text
        let problems = doc.contrast_problems();
        let summary: Vec<_> = problems.iter().map(|p| (p.role, p.fg_name.unwrap(), p.bg_name.unwrap(), p.cells.clone())).collect();
        assert_eq!(
            summary,
            vec![
                (Role::Text, "Light gray", "Light gray", vec![6]),
                (Role::Text, "Light red", "Blue", vec![0, 1]),
                (Role::Graphic, "Blue", "Black", vec![4]),
            ]
        );
        assert_eq!(format!("{:.2}", problems[1].ratio), "4.23");
        assert_eq!(problems[1].needs, TEXT_RATIO);
        assert_eq!(problems[2].needs, GRAPHIC_RATIO);
    }

    #[test]
    fn bold_counts_as_bright() {
        // Bold black is drawn dark gray: 3.21:1 on light gray, where black
        // would pass at 9.04:1.
        let doc = Document::open("ans", b"\x1b[1;30;47mA").unwrap();
        let problems = doc.contrast_problems();
        assert_eq!(problems.len(), 1);
        assert_eq!((problems[0].fg_name, problems[0].bg_name), (Some("Dark gray"), Some("Light gray")));
    }
}
