//! PNG export (docs/roadmap.md, Phase 1c): the art at its native pixel
//! size, in 8- or 9-px cells, optionally stretched to the original
//! display's aspect. Pixels come from the same renderer as the viewer.
//!
//! The export is written a band of rows at a time, so the caller can show a
//! determinate progress bar over rows and a long ANSI never has to be one
//! image in memory. Aspect correction resamples rows with area weights in
//! integer maths, so an export is the same on every machine.

use std::io::Write;

use serde::{Deserialize, Serialize};

use crate::document::Document;

/// How to export; starts from the document's render settings. iCE colors
/// always follow the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PngOptions {
    /// 9-px cells (only fonts 8 px wide have a 9th column to add).
    pub letter_spacing: bool,
    /// Stretch vertically to the original display's aspect.
    pub aspect_ratio: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PngSize {
    pub width: u32,
    pub height: u32,
}

impl Document {
    /// The PNG's pixel size with these options, and the rendered (unstretched) height.
    fn png_geometry(&mut self, options: PngOptions) -> (PngSize, u32) {
        self.with_letter_spacing(options.letter_spacing, |doc| {
            let info = doc.info();
            let height = info.pixel_height.max(0) as u32;
            let stretch = doc.buffer.get_aspect_ratio_stretch_factor();
            let out = if options.aspect_ratio && stretch > 0.0 {
                (height as f64 * stretch as f64).round() as u32
            } else {
                height
            };
            (PngSize { width: info.pixel_width.max(0) as u32, height: out }, height)
        })
    }

    /// The size a PNG export would be, for the export dialog.
    pub fn png_size(&mut self, options: PngOptions) -> PngSize {
        self.png_geometry(options).0
    }
}

/// A PNG export under way: begun, fed bands with `write_rows` until
/// `is_done`, then `finish`ed.
pub struct PngExport<W: Write + 'static> {
    writer: png::StreamWriter<'static, W>,
    options: PngOptions,
    /// Rendered height and output height in pixels, and bytes per pixel row.
    source_height: u64,
    out_height: u64,
    stride: usize,
    /// Art rows (cells) in total and written so far.
    rows: i32,
    rows_done: i32,
    /// Rendered pixel rows not yet used up, from pixel row `held_first`.
    held: Vec<u8>,
    held_first: u64,
    /// The next output pixel row.
    next_out: u64,
}

impl<W: Write + 'static> PngExport<W> {
    pub fn begin(doc: &mut Document, options: PngOptions, out: W) -> Result<Self, String> {
        let (size, source_height) = doc.png_geometry(options);
        if size.width == 0 || size.height == 0 {
            return Err("There's no art to export".into());
        }
        let mut encoder = png::Encoder::new(out, size.width, size.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let writer = encoder.write_header().and_then(|w| w.into_stream_writer()).map_err(|e| e.to_string())?;
        Ok(Self {
            writer,
            options,
            source_height: source_height as u64,
            out_height: size.height as u64,
            stride: size.width as usize * 4,
            rows: doc.rows(),
            rows_done: 0,
            held: Vec::new(),
            held_first: 0,
            next_out: 0,
        })
    }

    /// Art rows in total, for the progress bar.
    pub fn rows(&self) -> i32 {
        self.rows
    }

    pub fn rows_done(&self) -> i32 {
        self.rows_done
    }

    pub fn is_done(&self) -> bool {
        self.next_out == self.out_height
    }

    /// Renders and writes the next `count` art rows. Returns the rows done.
    pub fn write_rows(&mut self, doc: &mut Document, count: i32) -> Result<i32, String> {
        if doc.rows() != self.rows {
            return Err("The art changed size during the export".into());
        }
        let count = count.clamp(0, self.rows - self.rows_done);
        if count > 0 {
            let band = doc.with_letter_spacing(self.options.letter_spacing, |doc| doc.render_rows(self.rows_done, count, true));
            if band.width as usize * 4 != self.stride {
                return Err("The art changed size during the export".into());
            }
            self.held.extend_from_slice(&band.rgba);
            self.rows_done += count;
        }
        self.emit()?;
        Ok(self.rows_done)
    }

    /// Writes every output row whose source rows have been rendered, then
    /// lets go of the rendered rows no later output row needs.
    fn emit(&mut self) -> Result<(), String> {
        let (h, out_h) = (self.source_height, self.out_height);
        let available = self.held_first + (self.held.len() / self.stride) as u64;
        let mut row = vec![0u8; self.stride];
        let mut sums = vec![0u64; self.stride];
        while self.next_out < out_h {
            // Output row y covers [y·h, (y+1)·h) and source row s covers
            // [s·out_h, (s+1)·out_h): both in units of 1 / (h · out_h).
            let y = self.next_out;
            let (lo, hi) = (y * h, (y + 1) * h);
            let first = lo / out_h;
            let end = hi.div_ceil(out_h);
            if end > available {
                break;
            }
            sums.iter_mut().for_each(|s| *s = 0);
            for s in first..end {
                let weight = hi.min((s + 1) * out_h) - lo.max(s * out_h);
                let at = (s - self.held_first) as usize * self.stride;
                for (sum, &p) in sums.iter_mut().zip(&self.held[at..at + self.stride]) {
                    *sum += weight * p as u64;
                }
            }
            // The weights add up to h.
            for (out, &sum) in row.iter_mut().zip(&sums) {
                *out = ((sum + h / 2) / h) as u8;
            }
            self.writer.write_all(&row).map_err(|e| e.to_string())?;
            self.next_out += 1;
        }
        let keep_from = if self.next_out < out_h { (self.next_out * h / out_h).min(available) } else { available };
        let drop = (keep_from - self.held_first) as usize * self.stride;
        self.held.drain(..drop);
        self.held_first = keep_from;
        Ok(())
    }

    /// Ends the PNG: the last of the image data and the IEND chunk. The
    /// writer is dropped, so a caller that must sync a file keeps a handle.
    pub fn finish(self) -> Result<(), String> {
        if !self.is_done() {
            return Err("The export isn't finished".into());
        }
        self.writer.finish().map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderSettings;

    /// A writer the test can read back after the export drops its copy.
    #[derive(Clone, Default)]
    struct Shared(std::rc::Rc<std::cell::RefCell<Vec<u8>>>);

    impl Write for Shared {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.borrow_mut().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn export(doc: &mut Document, options: PngOptions, band: i32) -> Vec<u8> {
        let out = Shared::default();
        let mut export = PngExport::begin(doc, options, out.clone()).unwrap();
        while !export.is_done() {
            export.write_rows(doc, band).unwrap();
        }
        export.finish().unwrap();
        out.0.take()
    }

    fn decode(png_bytes: &[u8]) -> (u32, u32, Vec<u8>) {
        let decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
        let mut reader = decoder.read_info().unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut pixels).unwrap();
        pixels.truncate(info.buffer_size());
        (info.width, info.height, pixels)
    }

    fn art() -> Document {
        Document::open("ans", b"\x1b[1;33;44mSTYLUS\x1b[0m\r\n\xdb\xdc\xdf\r\ntwo\r\n\x1b[31mthree").unwrap()
    }

    #[test]
    fn exports_the_viewer_pixels_at_native_size() {
        let mut doc = art();
        doc.set_settings(RenderSettings { letter_spacing: false, aspect_ratio: false, ..doc.settings() });
        let rendered = doc.render_rows(0, doc.rows(), true);
        // Bands of one row come out the same as the whole art at once.
        let (w, h, pixels) = decode(&export(&mut doc, PngOptions { letter_spacing: false, aspect_ratio: false }, 1));
        assert_eq!((w, h), (rendered.width, rendered.height));
        assert_eq!(pixels, rendered.rgba);
        assert_eq!(doc.png_size(PngOptions { letter_spacing: false, aspect_ratio: false }), PngSize { width: w, height: h });
    }

    #[test]
    fn nine_pixel_cells_without_changing_the_document() {
        let mut doc = art();
        doc.set_settings(RenderSettings { letter_spacing: false, ..doc.settings() });
        let (w, _, _) = decode(&export(&mut doc, PngOptions { letter_spacing: true, aspect_ratio: false }, 2));
        assert_eq!(w, 80 * 9);
        assert!(!doc.settings().letter_spacing, "the view is left as it was");
    }

    #[test]
    fn aspect_correction_stretches_rows_the_same_in_any_band_size() {
        let mut doc = art();
        let options = PngOptions { letter_spacing: false, aspect_ratio: true };
        let whole = export(&mut doc, options, 1000);
        let (w, h, pixels) = decode(&whole);
        let rows = doc.rows() as u32;
        assert_eq!((w, h), (640, (rows as f64 * 16.0 * 1.2).round() as u32));
        assert_eq!(doc.png_size(options).height, h);
        for band in [1, 2, 3] {
            assert_eq!(export(&mut doc, options, band), whole, "band size {band}");
        }
        // A solid area stays exactly its color: the top-left of the blue row.
        let blue = doc.render_rows(0, 1, true).rgba[..4].to_vec();
        assert_eq!(&pixels[..4], &blue[..]);
    }
}
