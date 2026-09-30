//! Image to ANSI: every open-source image-to-ANSI converter whose license
//! allows use in closed-source commercial software, ported to Rust so they
//! run here, in the browser build too, with nothing sent anywhere.
//!
//! Each converter is identified by where it came from (its project), and
//! its module ports that project's algorithm at the revision recorded in
//! its `ConverterInfo`, with the settings named there. What every port
//! changes, and why, is in `ConverterInfo::adaptations`. The survey of
//! converters found, including the ones left out and why, is
//! docs/image-to-ansi-converters.md.
//!
//! All of them run at the same width (`COLUMNS`) and their cells go
//! through one .ANS writer (`grid::write_ans`), so a difference between two
//! outputs is a difference between the converters.

mod grid;
mod magick;
mod nfnt;
mod stbir;
mod util;

mod aimg;
mod ansi_art;
mod ansify;
mod ansipx;
mod ansir;
mod ansize;
mod ascii_image_converter;
mod ascii_magic;
mod asciimatics;
mod catimg;
mod chromatic;
mod climage;
mod eddieantonio_imgcat;
mod hiptext;
mod image_to_ansi;
mod image_to_ascii;
mod img2ansi_bmatsuo;
mod img2ansi_jakobwesthoff;
mod img2ansi_johnmccabe;
mod img2ansi_lloiser;
mod img2ansi_wbrown;
mod img2txt;
mod img_to_txt;
mod koan_ansi;
mod libcaca;
mod png2ansi_1hyena;
mod png2ansi_theomat;
mod ransid;
mod rich_pixels;
mod tapciify;
mod term_image;
mod terminal_image;
mod termpix;
mod tiv;
mod trashhalo_imgcat;
mod viuer;

pub use grid::{AnsMeta, Cell, Color, Grid};

pub use image::RgbaImage;
use serde::Serialize;

/// Output width in columns, for every converter: the classic ANSI width.
pub const COLUMNS: u32 = 80;

/// The largest image Stylus converts, in pixels, so a huge photo can't
/// stall a converter. Bigger images are shrunk first (box filter), which
/// every converter would do anyway on the way down to 80 columns.
const MAX_PIXELS: u32 = 4096 * 4096;

/// File extensions the Image to ANSI panel takes (what the `image` crate
/// decodes here). Lower case, without the dot.
pub const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "bmp", "webp", "tif", "tiff", "tga", "ico", "pbm", "pgm", "ppm", "pnm", "qoi"];

/// Where a converter came from and how it's run here.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConverterInfo {
    /// Stable id, used in saved file names.
    pub id: &'static str,
    /// The project's own name.
    pub name: &'static str,
    /// Where it's from: the repository, e.g. "github.com/koan-shdw/koan-ansi".
    pub origin: &'static str,
    /// The copyright line from its license.
    pub copyright: &'static str,
    /// SPDX license identifier (the permissive option, for dual licenses).
    pub license: &'static str,
    /// The language of the original.
    pub language: &'static str,
    /// The commit ported.
    pub revision: &'static str,
    /// The original's settings used here, in its own terms.
    pub settings: &'static str,
    /// What the port changes from the original, and why.
    pub adaptations: &'static str,
    /// The license's full text, which its terms require shipping.
    #[serde(skip)]
    pub license_text: &'static str,
}

/// Library code the ports carry (resamplers and color math their originals
/// get from these libraries), with the license texts that must ship.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryNotice {
    pub name: &'static str,
    pub origin: &'static str,
    pub license: &'static str,
    /// Which parts are ported, and for which converters.
    pub used_for: &'static str,
    pub license_text: &'static str,
}

pub const LIBRARY_NOTICES: &[LibraryNotice] = &[
    LibraryNotice {
        name: "Pillow",
        origin: "github.com/python-pillow/Pillow",
        license: "MIT-CMU",
        used_for: "Image.resize's resampling (util::resample, util::pillow_nearest), alpha_composite (util::pillow_over) and its RGB-to-palette conversion, for the Python converters",
        license_text: include_str!("licenses/libraries/pillow.txt"),
    },
    LibraryNotice {
        name: "Go x/image/draw",
        origin: "golang.org/x/image",
        license: "BSD-3-Clause",
        used_for: "the kernel scaler, for img2ansi (Wes Brown)",
        license_text: include_str!("licenses/libraries/x-image.txt"),
    },
    LibraryNotice {
        name: "Go standard library",
        origin: "go.dev",
        license: "BSD-3-Clause",
        used_for: "math.Sin (util::go_sin), color.Palette.Index and the color models, for the Go converters",
        license_text: include_str!("licenses/libraries/go.txt"),
    },
    LibraryNotice {
        name: "stb_image_resize2",
        origin: "github.com/nothings/stb",
        license: "MIT",
        used_for: "stbir_resize_uint8_linear's resize (stbir.rs), for img_to_txt",
        license_text: include_str!("licenses/libraries/stb-image-resize2.txt"),
    },
    LibraryNotice {
        name: "ImageMagick",
        origin: "github.com/ImageMagick/ImageMagick",
        license: "ImageMagick",
        used_for: "ResizeImage with its Lanczos and Mitchell filters (magick.rs), for ransid",
        license_text: include_str!("licenses/libraries/imagemagick.txt"),
    },
    LibraryNotice {
        name: "OpenCV",
        origin: "github.com/opencv/opencv",
        license: "Apache-2.0",
        used_for: "RGB-to-gray, the bit-exact Gaussian kernel (with its exp), filter2D and linear resize, for chromatic",
        license_text: include_str!("licenses/libraries/opencv.txt"),
    },
    LibraryNotice {
        name: "go-colorful",
        origin: "github.com/lucasb-eyer/go-colorful",
        license: "MIT",
        used_for: "MakeColor and RGBA, for ansipx",
        license_text: include_str!("licenses/libraries/go-colorful.txt"),
    },
    LibraryNotice {
        name: "lwip",
        origin: "github.com/EyalAr/lwip",
        license: "MIT",
        used_for: "loading to RGBA (alpha as a percentage) and resize, for image-to-ascii",
        license_text: include_str!("licenses/libraries/lwip.txt"),
    },
    LibraryNotice {
        name: "nfnt/resize",
        origin: "github.com/nfnt/resize",
        license: "ISC",
        used_for: "Resize and Thumbnail, for ansize, img2ansi (Bryan Matsuo, Lukas Beranek), imgcat (Stephen Solka) and ansipx",
        license_text: include_str!("licenses/libraries/nfnt-resize.txt"),
    },
    LibraryNotice {
        name: "imaging",
        origin: "github.com/disintegration/imaging",
        license: "MIT",
        used_for: "Resize with Lanczos, for ascii-image-converter and ANSI-art",
        license_text: include_str!("licenses/libraries/imaging.txt"),
    },
    LibraryNotice {
        name: "kdtree",
        origin: "github.com/stefankoegl/kdtree",
        license: "ISC",
        used_for: "the nearest-color search, for CLImage",
        license_text: include_str!("licenses/libraries/kdtree.txt"),
    },
    LibraryNotice {
        name: "chroma.js",
        origin: "github.com/gka/chroma.js",
        license: "BSD-3-Clause",
        used_for: "HSL mixing and CIE Lab, for ansir",
        license_text: include_str!("licenses/libraries/chroma-js.txt"),
    },
    LibraryNotice {
        name: "Jimp",
        origin: "github.com/jimp-dev/jimp",
        license: "MIT",
        used_for: "its default resize, for terminal-image",
        license_text: include_str!("licenses/libraries/jimp.txt"),
    },
];

pub trait Converter: Sync {
    fn info(&self) -> &'static ConverterInfo;
    /// Converts an image (straight, non-premultiplied RGBA) to cells,
    /// `columns` wide.
    fn convert(&self, image: &RgbaImage, columns: u32) -> Grid;
}

/// Every converter, in the order the panel shows them.
pub fn converters() -> &'static [&'static dyn Converter] {
    &[
        // Made for scene-style 16-color ANSI art.
        &koan_ansi::KoanAnsi,
        &img2ansi_wbrown::Img2AnsiWbrown,
        &libcaca::Libcaca,
        &png2ansi_1hyena::Png2Ansi1Hyena,
        &ansify::Ansify,
        &ansir::Ansir,
        &climage::Climage,
        &tiv::Tiv,
        // Terminal image viewers: half blocks.
        &viuer::Viuer,
        &catimg::Catimg,
        &terminal_image::TerminalImage,
        &trashhalo_imgcat::TrashhaloImgcat,
        &eddieantonio_imgcat::EddieantonioImgcat,
        &termpix::Termpix,
        &image_to_ansi::ImageToAnsi,
        &aimg::Aimg,
        &img2ansi_johnmccabe::Img2AnsiJohnmccabe,
        &img2ansi_jakobwesthoff::Img2AnsiJakobwesthoff,
        &ansipx::Ansipx,
        &asciimatics::Asciimatics,
        &term_image::TermImage,
        &rich_pixels::RichPixels,
        // One colored space per pixel.
        &img2txt::Img2txt,
        &img2ansi_bmatsuo::Img2AnsiBmatsuo,
        &hiptext::Hiptext,
        &png2ansi_theomat::Png2AnsiTheomat,
        &ransid::Ransid,
        // Colored ASCII.
        &ansize::Ansize,
        &ascii_image_converter::AsciiImageConverter,
        &ascii_magic::AsciiMagic,
        &img2ansi_lloiser::Img2AnsiLloiser,
        &tapciify::Tapciify,
        &image_to_ascii::ImageToAscii,
        &img_to_txt::ImgToTxt,
        &chromatic::Chromatic,
        &ansi_art::AnsiArt,
    ]
}

pub fn converter(id: &str) -> Option<&'static dyn Converter> {
    converters().iter().copied().find(|c| c.info().id == id)
}

/// Decodes an image file's bytes. `extension` (without the dot) picks the
/// format when the bytes don't say.
pub fn decode_image(extension: &str, data: &[u8]) -> Result<RgbaImage, String> {
    let format = image::ImageFormat::from_extension(extension);
    let reader = image::ImageReader::new(std::io::Cursor::new(data));
    let reader = match reader.with_guessed_format() {
        Ok(r) if r.format().is_some() => r,
        _ => {
            let mut r = image::ImageReader::new(std::io::Cursor::new(data));
            if let Some(f) = format {
                r.set_format(f);
            }
            r
        }
    };
    let img = reader.decode().map_err(|e| format!("Couldn't read the image: {e}"))?.to_rgba8();
    if img.width() == 0 || img.height() == 0 {
        return Err("The image is empty".to_string());
    }
    let (w, h) = img.dimensions();
    if (w as u64) * (h as u64) > MAX_PIXELS as u64 {
        let s = (MAX_PIXELS as f64 / (w as f64 * h as f64)).sqrt();
        let (nw, nh) = (((w as f64 * s) as u32).max(1), ((h as f64 * s) as u32).max(1));
        return Ok(util::resample(&img, nw, nh, util::Kernel::Box));
    }
    Ok(img)
}

/// Runs one converter and writes its .ANS file. `title` goes in SAUCE (the
/// image's name), `date` is `CCYYMMDD`.
pub fn convert_to_ans(converter: &dyn Converter, image: &RgbaImage, title: &str, date: &str, app_version: &str) -> Vec<u8> {
    let info = converter.info();
    let grid = converter.convert(image, COLUMNS);
    let mut comments = vec![
        format!("Converted by Stylus {app_version} with {}", info.name),
        format!("from {}", info.origin),
        format!("{} license. {}", info.license, info.copyright),
    ];
    comments.extend(wrap(&format!("Settings: {}", info.settings), 64));
    write_ans(&grid, title, date, &comments)
}

fn write_ans(grid: &Grid, title: &str, date: &str, comments: &[String]) -> Vec<u8> {
    grid::write_ans(grid, &AnsMeta { title, date, comments })
}

/// Splits text into lines of at most `width` characters, at spaces.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use image::Rgba;

    /// A small test image: a hue sweep across, dark to light down, with a
    /// transparent corner.
    pub(crate) fn test_image(w: u32, h: u32) -> RgbaImage {
        RgbaImage::from_fn(w, h, |x, y| {
            let t = x as f64 / w as f64;
            let v = y as f64 / h as f64;
            let r = ((t * 6.0).sin().abs() * 255.0 * v) as u8;
            let g = (((t * 6.0) + 2.0).sin().abs() * 255.0 * v) as u8;
            let b = (((t * 6.0) + 4.0).sin().abs() * 255.0 * v) as u8;
            let a = if x < w / 8 && y < h / 8 { 0 } else { 255 };
            Rgba([r, g, b, a])
        })
    }

    #[test]
    fn ids_are_unique() {
        let mut ids: Vec<&str> = converters().iter().map(|c| c.info().id).collect();
        let n = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), n);
    }

    #[test]
    fn every_converter_makes_an_ans_stylus_opens_at_80_columns() {
        let img = test_image(160, 120);
        for c in converters() {
            let info = c.info();
            let grid = c.convert(&img, COLUMNS);
            // Some originals round the width down by one (catimg's float
            // scale, for one); none goes past 80.
            assert!((COLUMNS as usize - 1..=COLUMNS as usize).contains(&grid.width), "{} is {} wide", info.id, grid.width);
            assert!(grid.height > 0, "{}", info.id);
            assert!(grid.cells.iter().all(|cell| cell.ch >= 0x20 && cell.ch != 0x7f), "{} wrote a control code", info.id);
            let ans = convert_to_ans(*c, &img, "test", "20260929", "0.0.0");
            let doc = Document::open("ans", &ans).unwrap_or_else(|e| panic!("{}: {e}", info.id));
            let doc_info = doc.info();
            assert_eq!(doc_info.columns, grid.width as i32, "{}", info.id);
            // The viewer leaves out trailing blank rows.
            assert!(doc_info.rows <= grid.height as i32, "{} rows", info.id);
            assert!(!info.license_text.is_empty(), "{} license text", info.id);
        }
    }

    #[test]
    fn conversions_are_reproducible() {
        let img = test_image(96, 64);
        for c in converters() {
            assert_eq!(c.convert(&img, COLUMNS).cells, c.convert(&img, COLUMNS).cells, "{}", c.info().id);
        }
    }

    #[test]
    fn tiny_and_extreme_images_convert() {
        for (w, h) in [(1, 1), (3, 60), (500, 2)] {
            let img = test_image(w, h);
            for c in converters() {
                let grid = c.convert(&img, COLUMNS);
                assert!(grid.height >= 1 && (1..=COLUMNS as usize).contains(&grid.width), "{} at {w}x{h}", c.info().id);
                assert_eq!(grid.cells.len(), grid.width * grid.height, "{} at {w}x{h}", c.info().id);
            }
        }
    }

    #[test]
    fn library_notices_have_texts() {
        assert!(LIBRARY_NOTICES.iter().all(|n| n.license_text.len() > 200), "a notice is missing its text");
    }

    #[test]
    fn wraps_at_spaces() {
        assert_eq!(wrap("aa bb cc", 5), vec!["aa bb", "cc"]);
    }
}
