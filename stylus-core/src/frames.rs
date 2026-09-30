//! Frames and layers in the document model (docs/roadmap.md, 1b): the
//! model carries both from the start, so the Phase 3 timeline and Layers
//! tab need no change to it or to the recovery snapshot.
//!
//! A document is frames × layers, as in a sprite editor: the layers (their
//! names, order and visibility) are the same in every frame, and each frame
//! has its own cells on each. The shown frame's layers are icy_engine's
//! `buffer.layers`, so everything that draws or edits works on the frame on
//! screen without knowing about the others; the rest wait in
//! `Document::frames`. Edits go to the current layer.
//!
//! Changing the frames or layers (and resizing, which touches every frame)
//! is one undo step holding the whole structure before and after.
//! Choosing which frame is shown or which layer is edited isn't an edit.

use icy_engine::{AttributedChar, Layer, Size, TextAttribute, TextPane};
use std::collections::HashSet;

use crate::document::Document;
use crate::edit::{CellRect, Step};

/// A new frame's hold time, as `docs/ui-design.md`'s timeline shows it.
pub const DEFAULT_HOLD_MS: u32 = 100;
/// The hold times a frame can have.
pub const MIN_HOLD_MS: u32 = 10;
pub const MAX_HOLD_MS: u32 = 60_000;
/// Most frames and layers a document can have.
pub const MAX_FRAMES: usize = 1000;
pub const MAX_LAYERS: usize = 100;

/// Sets a cell whether or not the layer is hidden or locked, for the
/// model's own changes (undo, the iCE switch, resizing): icy_engine's
/// `set_char` quietly skips hidden and locked layers, which is right only
/// for the user's own drawing.
pub(crate) fn put(layer: &mut Layer, x: i32, y: i32, ch: AttributedChar) {
    let properties = layer.properties.clone();
    layer.properties.is_visible = true;
    layer.properties.is_locked = false;
    layer.properties.is_alpha_channel_locked = false;
    layer.set_char((x, y), ch);
    layer.properties = properties;
}

#[derive(Debug, Clone)]
pub(crate) struct Frame {
    /// Empty for the shown frame, whose layers are in the buffer.
    pub layers: Vec<Layer>,
    /// How long the frame shows when played, in milliseconds.
    pub hold_ms: u32,
}

/// Everything a structural change can touch, for its undo step.
#[derive(Clone)]
pub(crate) struct Structure {
    size: Size,
    rows: i32,
    /// Every frame with its layers, the shown one's included.
    frames: Vec<Frame>,
    frame: usize,
    layer: usize,
    switched: HashSet<(usize, usize, i32, i32)>,
    ice: bool,
}

impl Document {
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn current_frame(&self) -> usize {
        self.frame
    }

    pub fn frame_hold(&self, frame: usize) -> Option<u32> {
        self.frames.get(frame).map(|f| f.hold_ms)
    }

    pub fn layer_count(&self) -> usize {
        self.buffer.layers.len()
    }

    pub fn current_layer(&self) -> usize {
        self.layer
    }

    /// Shows frame `frame`. Not an edit.
    pub fn select_frame(&mut self, frame: usize) -> Result<CellRect, String> {
        self.check_frame(frame)?;
        self.show_frame(frame);
        Ok(self.whole())
    }

    /// Makes edits go to layer `layer`. Not an edit.
    pub fn select_layer(&mut self, layer: usize) -> Result<(), String> {
        self.check_layer(layer)?;
        self.layer = layer;
        Ok(())
    }

    /// Adds a frame at `at` (0 to the frame count) and shows it: a copy of
    /// the frame shown, or blank. Undoable.
    pub fn insert_frame(&mut self, at: usize, copy: bool) -> Result<CellRect, String> {
        if at > self.frames.len() {
            return Err(format!("There's no place {} in {} frames", at + 1, self.frames.len()));
        }
        if self.frames.len() >= MAX_FRAMES {
            return Err(format!("A document can have at most {MAX_FRAMES} frames"));
        }
        self.change_structure(|doc| {
            let layers = if copy { doc.buffer.layers.clone() } else { doc.blank_layers() };
            let hold_ms = doc.frames[doc.frame].hold_ms;
            doc.frames.insert(at, Frame { layers, hold_ms });
            doc.remap_frames(|f| Some(if f >= at { f + 1 } else { f }));
            if doc.frame >= at {
                doc.frame += 1;
            }
            doc.show_frame(at);
        });
        Ok(self.whole())
    }

    /// Deletes frame `frame`; the next one (or the last) is shown if it was.
    /// The only frame can't go. Undoable.
    pub fn delete_frame(&mut self, frame: usize) -> Result<CellRect, String> {
        self.check_frame(frame)?;
        if self.frames.len() == 1 {
            return Err("A document keeps at least one frame".to_string());
        }
        self.change_structure(|doc| {
            if doc.frame == frame {
                doc.show_frame(if frame + 1 < doc.frames.len() { frame + 1 } else { frame - 1 });
            }
            doc.frames.remove(frame);
            doc.remap_frames(|f| match f.cmp(&frame) {
                std::cmp::Ordering::Less => Some(f),
                std::cmp::Ordering::Equal => None,
                std::cmp::Ordering::Greater => Some(f - 1),
            });
            if doc.frame > frame {
                doc.frame -= 1;
            }
        });
        Ok(self.whole())
    }

    /// Moves frame `from` to place `to`, the others closing up. Undoable.
    pub fn move_frame(&mut self, from: usize, to: usize) -> Result<(), String> {
        self.check_frame(from)?;
        self.check_frame(to)?;
        if from == to {
            return Ok(());
        }
        self.change_structure(|doc| {
            let moved = doc.frames.remove(from);
            doc.frames.insert(to, moved);
            let place = |f: usize| {
                if f == from {
                    to
                } else if from < f && f <= to {
                    f - 1
                } else if to <= f && f < from {
                    f + 1
                } else {
                    f
                }
            };
            doc.remap_frames(|f| Some(place(f)));
            doc.frame = place(doc.frame);
        });
        Ok(())
    }

    /// Sets how long frame `frame` shows when played. Undoable.
    pub fn set_frame_hold(&mut self, frame: usize, hold_ms: u32) -> Result<(), String> {
        self.check_frame(frame)?;
        if !(MIN_HOLD_MS..=MAX_HOLD_MS).contains(&hold_ms) {
            return Err(format!("A frame can show for {MIN_HOLD_MS} ms to {} seconds", MAX_HOLD_MS / 1000));
        }
        if self.frames[frame].hold_ms != hold_ms {
            self.change_structure(|doc| doc.frames[frame].hold_ms = hold_ms);
        }
        Ok(())
    }

    /// Adds an empty layer (every cell transparent) above the current one,
    /// in every frame, and makes it current. Undoable.
    pub fn add_layer(&mut self, title: &str) -> Result<(), String> {
        if self.buffer.layers.len() >= MAX_LAYERS {
            return Err(format!("A document can have at most {MAX_LAYERS} layers"));
        }
        let at = self.layer + 1;
        let size = self.buffer.size();
        self.change_structure(|doc| {
            for f in 0..doc.frames.len() {
                let mut layer = Layer::new(title, size);
                layer.properties.has_alpha_channel = true;
                doc.frame_layers_mut(f).insert(at, layer);
            }
            doc.remap_layers(|l| Some(if l >= at { l + 1 } else { l }));
            doc.layer = at;
        });
        Ok(())
    }

    /// Deletes layer `layer` from every frame; the one below (or the new
    /// bottom) becomes current if it was. The only layer can't go. Undoable.
    pub fn delete_layer(&mut self, layer: usize) -> Result<CellRect, String> {
        self.check_layer(layer)?;
        if self.buffer.layers.len() == 1 {
            return Err("A document keeps at least one layer".to_string());
        }
        self.change_structure(|doc| {
            for f in 0..doc.frames.len() {
                doc.frame_layers_mut(f).remove(layer);
            }
            doc.remap_layers(|l| match l.cmp(&layer) {
                std::cmp::Ordering::Less => Some(l),
                std::cmp::Ordering::Equal => None,
                std::cmp::Ordering::Greater => Some(l - 1),
            });
            if doc.layer >= layer && doc.layer > 0 {
                doc.layer -= 1;
            }
        });
        Ok(self.whole())
    }

    /// Shows or hides layer `layer` in every frame. Undoable.
    pub fn set_layer_visible(&mut self, layer: usize, visible: bool) -> Result<CellRect, String> {
        self.check_layer(layer)?;
        if self.buffer.layers[layer].is_visible() != visible {
            self.change_structure(|doc| {
                for f in 0..doc.frames.len() {
                    doc.frame_layers_mut(f)[layer].set_is_visible(visible);
                }
            });
        }
        Ok(self.whole())
    }

    fn check_frame(&self, frame: usize) -> Result<(), String> {
        if frame < self.frames.len() {
            Ok(())
        } else {
            Err(format!("There's no frame {} in {}", frame + 1, self.frames.len()))
        }
    }

    fn check_layer(&self, layer: usize) -> Result<(), String> {
        if layer < self.buffer.layers.len() {
            Ok(())
        } else {
            Err(format!("There's no layer {} in {}", layer + 1, self.buffer.layers.len()))
        }
    }

    /// The whole canvas, to redraw after the frame shown changes.
    pub(crate) fn whole(&self) -> CellRect {
        CellRect { x: 0, y: 0, width: self.buffer.width(), height: self.rows }
    }

    /// Puts frame `frame`'s layers in the buffer, and the shown one's back.
    pub(crate) fn show_frame(&mut self, frame: usize) {
        if frame == self.frame {
            return;
        }
        self.frames[self.frame].layers = std::mem::take(&mut self.buffer.layers);
        self.buffer.layers = std::mem::take(&mut self.frames[frame].layers);
        self.frame = frame;
        self.buffer.mark_dirty();
    }

    /// The shown frame's layers with nothing drawn: blank on the bottom
    /// layer (spaces, light gray on black, as a new document), transparent
    /// above it.
    fn blank_layers(&self) -> Vec<Layer> {
        let size = self.buffer.size();
        let blank = AttributedChar::new(' ', TextAttribute::new(7, 0));
        self.buffer
            .layers
            .iter()
            .enumerate()
            .map(|(i, source)| {
                let mut layer = Layer::new(source.title(), size);
                layer.role = source.role;
                layer.properties = source.properties.clone();
                if i == 0 {
                    for y in 0..size.height {
                        for x in 0..size.width {
                            put(&mut layer, x, y, blank);
                        }
                    }
                }
                layer
            })
            .collect()
    }

    fn remap_frames(&mut self, map: impl Fn(usize) -> Option<usize>) {
        self.switched = self.switched.drain().filter_map(|(f, l, x, y)| map(f).map(|f| (f, l, x, y))).collect();
    }

    fn remap_layers(&mut self, map: impl Fn(usize) -> Option<usize>) {
        self.switched = self.switched.drain().filter_map(|(f, l, x, y)| map(l).map(|l| (f, l, x, y))).collect();
    }

    pub(crate) fn structure(&self) -> Structure {
        let mut frames = self.frames.clone();
        frames[self.frame].layers = self.buffer.layers.clone();
        Structure {
            size: self.buffer.size(),
            rows: self.rows,
            frames,
            frame: self.frame,
            layer: self.layer,
            switched: self.switched.clone(),
            ice: self.ice,
        }
    }

    /// Puts back a structure from an undo step. If iCE was switched since,
    /// the restored cells are switched the same way.
    pub(crate) fn set_structure(&mut self, structure: &Structure) -> CellRect {
        let ice = self.ice;
        let mut frames = structure.frames.clone();
        self.buffer.set_size(structure.size);
        self.buffer.layers = std::mem::take(&mut frames[structure.frame].layers);
        self.frames = frames;
        self.frame = structure.frame;
        self.layer = structure.layer;
        self.rows = structure.rows;
        self.switched = structure.switched.clone();
        self.ice = structure.ice;
        self.buffer.ice_mode = if self.ice { icy_engine::IceMode::Ice } else { icy_engine::IceMode::Blink };
        if ice != self.ice {
            self.switch_ice_colors(ice);
        }
        self.buffer.mark_dirty();
        self.whole()
    }

    /// Runs a change to the frames or layers as one undo step.
    pub(crate) fn change_structure(&mut self, change: impl FnOnce(&mut Self)) {
        let before = self.structure();
        change(self);
        self.buffer.mark_dirty();
        let after = self.structure();
        self.push_step(Step::Structure { before: Box::new(before), after: Box::new(after) });
    }
}

#[cfg(test)]
mod tests {
    use crate::{CellEdit, Document, RenderSettings};

    fn edit(x: i32, y: i32, code: u8, fg: u8, bg: u8) -> CellEdit {
        CellEdit { x, y, code: Some(code), fg: Some(fg), bg: Some(bg) }
    }

    fn code(doc: &Document, x: i32, y: i32) -> u8 {
        doc.cell(x, y).unwrap().code as u8
    }

    #[test]
    fn a_document_starts_with_one_frame_and_one_layer() {
        let info = Document::new_blank(10, 4, true).unwrap().info();
        assert_eq!((info.frames, info.frame, info.layers, info.layer), (1, 0, 1, 0));
    }

    #[test]
    fn frames_hold_their_own_art() {
        let mut doc = Document::new_blank(10, 4, true).unwrap();
        doc.apply(1, &[edit(0, 0, b'a', 15, 1)]);
        doc.insert_frame(1, true).unwrap();
        assert_eq!(doc.current_frame(), 1);
        assert_eq!(code(&doc, 0, 0), b'a', "a copy");
        doc.apply(2, &[edit(0, 0, b'b', 15, 1)]);
        doc.insert_frame(2, false).unwrap();
        assert_eq!(doc.cell(0, 0).unwrap().code, 32, "blank");
        doc.select_frame(0).unwrap();
        assert_eq!(code(&doc, 0, 0), b'a');
        doc.select_frame(1).unwrap();
        assert_eq!(code(&doc, 0, 0), b'b');
        assert_eq!(doc.info().frames, 3);
        assert!(doc.select_frame(3).is_err());
    }

    #[test]
    fn frame_changes_undo_and_redo() {
        let mut doc = Document::new_blank(10, 4, true).unwrap();
        doc.apply(1, &[edit(0, 0, b'a', 15, 1)]);
        doc.insert_frame(1, false).unwrap();
        doc.apply(2, &[edit(0, 0, b'b', 15, 1)]);
        doc.insert_frame(0, false).unwrap();
        doc.apply(3, &[edit(0, 0, b'c', 15, 1)]);
        // c, a, b
        doc.move_frame(0, 2).unwrap();
        // a, b, c; still showing c
        assert_eq!((doc.current_frame(), code(&doc, 0, 0)), (2, b'c'));
        doc.select_frame(0).unwrap();
        assert_eq!(code(&doc, 0, 0), b'a');
        doc.delete_frame(0).unwrap();
        assert_eq!((doc.frame_count(), code(&doc, 0, 0)), (2, b'b'));
        doc.set_frame_hold(1, 250).unwrap();
        assert!(doc.set_frame_hold(1, 0).is_err());

        doc.undo(); // hold
        assert_eq!(doc.frame_hold(1), Some(100));
        doc.undo(); // delete
        assert_eq!((doc.frame_count(), code(&doc, 0, 0)), (3, b'a'));
        doc.undo(); // move
        assert_eq!((doc.current_frame(), code(&doc, 0, 0)), (0, b'c'));
        doc.select_frame(2).unwrap();
        // Undoing a stroke on another frame shows that frame.
        let rect = doc.undo().unwrap();
        assert_eq!((doc.current_frame(), doc.cell(0, 0).unwrap().code), (0, 32));
        assert_eq!((rect.width, rect.height), (10, 4), "the whole frame to redraw");
        while doc.undo().is_some() {}
        assert_eq!((doc.frame_count(), doc.cell(0, 0).unwrap().code), (1, 32));
        assert!(!doc.is_edited());
        while doc.redo().is_some() {}
        assert_eq!((doc.frame_count(), doc.frame_hold(1)), (2, Some(250)));
        doc.select_frame(0).unwrap();
        assert_eq!(code(&doc, 0, 0), b'b');
        assert!(doc.delete_frame(0).is_ok() && doc.delete_frame(0).is_err(), "the last frame stays");
    }

    #[test]
    fn layers_are_the_same_in_every_frame() {
        let mut doc = Document::new_blank(10, 4, true).unwrap();
        doc.apply(1, &[edit(0, 0, b'a', 15, 1), edit(1, 0, b'a', 15, 1)]);
        doc.insert_frame(1, true).unwrap();
        doc.add_layer("Top").unwrap();
        assert_eq!((doc.layer_count(), doc.current_layer()), (2, 1));
        // Drawn on the top layer: covers the cell below, and the cell
        // beside it still shows the bottom layer through.
        doc.apply(2, &[edit(0, 0, b'T', 14, 4)]);
        assert_eq!((code(&doc, 0, 0), code(&doc, 1, 0)), (b'T', b'a'));
        doc.select_frame(0).unwrap();
        assert_eq!(doc.layer_count(), 2);
        assert_eq!(code(&doc, 0, 0), b'a');
        doc.select_frame(1).unwrap();
        doc.set_layer_visible(1, false).unwrap();
        assert_eq!(code(&doc, 0, 0), b'a', "hidden");
        assert_eq!(doc.apply(9, &[edit(5, 0, b'h', 1, 2)]), None, "a hidden layer isn't drawn on");
        doc.set_layer_visible(1, true).unwrap();
        doc.select_layer(0).unwrap();
        doc.apply(3, &[edit(0, 0, b'u', 14, 4)]);
        assert_eq!(code(&doc, 0, 0), b'T', "under the top layer");
        doc.delete_layer(1).unwrap();
        assert_eq!((code(&doc, 0, 0), doc.layer_count()), (b'u', 1));
        assert!(doc.delete_layer(0).is_err());
        doc.undo();
        assert_eq!((code(&doc, 0, 0), doc.layer_count()), (b'T', 2));
    }

    #[test]
    fn resizing_resizes_every_frame() {
        let mut doc = Document::new_blank(10, 4, true).unwrap();
        doc.insert_frame(1, false).unwrap();
        doc.resize(20, 6).unwrap();
        doc.select_frame(0).unwrap();
        assert_eq!(doc.cell(19, 5).unwrap().code, 32);
        doc.undo();
        assert_eq!((doc.columns(), doc.rows(), doc.frame_count()), (10, 4, 2));
    }

    #[test]
    fn ice_switches_every_frame_and_survives_undo() {
        let mut doc = Document::new_blank(4, 1, true).unwrap();
        doc.apply(1, &[edit(0, 0, b'x', 15, 12)]);
        doc.insert_frame(1, true).unwrap();
        let before = doc.render_rows(0, 1, true).rgba;
        doc.set_settings(RenderSettings { ice_colors: false, ..doc.settings() });
        doc.select_frame(0).unwrap();
        assert!(doc.cell(0, 0).unwrap().blink, "the other frame switched too");
        // Undoing the frame insert brings back cells stored with iCE on:
        // they're switched to match the setting now.
        doc.undo();
        assert!(doc.cell(0, 0).unwrap().blink);
        doc.set_settings(RenderSettings { ice_colors: true, ..doc.settings() });
        assert_eq!(doc.render_rows(0, 1, true).rgba, before);
    }

    #[test]
    fn saving_says_what_frames_and_layers_lose() {
        let mut doc = Document::new_blank(80, 2, true).unwrap();
        doc.apply(1, &[edit(0, 0, b'a', 15, 1), edit(1, 0, b'b', 15, 1)]);
        assert!(doc.save_losses("ans").unwrap().is_empty());
        doc.add_layer("Top").unwrap();
        doc.apply(2, &[edit(1, 0, b'T', 14, 4)]);
        doc.insert_frame(1, true).unwrap();
        doc.apply(3, &[edit(2, 0, b'2', 14, 4)]);
        let losses: Vec<String> = doc.save_losses("ans").unwrap().into_iter().map(|l| l.message).collect();
        assert!(losses.iter().any(|m| m.contains("frame 2 of 2")), "{losses:?}");
        assert!(losses.iter().any(|m| m.contains("2 layers are merged")), "{losses:?}");
        // The merged frame shown is what's saved (one row: a reopened ANSI
        // ends at its last row with content).
        let expected = doc.render_rows(0, 1, true).rgba;
        let saved = Document::open("ans", &doc.save("ans", &Default::default()).unwrap()).unwrap();
        assert_eq!(saved.render_rows(0, 1, true).rgba, expected);
        doc.set_layer_visible(1, false).unwrap();
        let losses: Vec<String> = doc.save_losses("ans").unwrap().into_iter().map(|l| l.message).collect();
        assert!(losses.iter().any(|m| m.contains("hidden")), "{losses:?}");
    }
}
