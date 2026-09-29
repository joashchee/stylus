//! Renders a file to PNG exactly as the viewer shows it (before aspect
//! correction, which is a display stretch). For scripts/compare-ansilove.sh.
//!
//! Usage: render_png <input> <output.png> [--8px|--9px] [--ice|--blink]
//! Without flags, the file's SAUCE (or icy_engine's defaults) decides.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use stylus_core::{Document, RenderSettings};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: render_png <input> <output.png> [--8px|--9px] [--ice|--blink]");
        std::process::exit(2);
    }
    let input = Path::new(&args[1]);
    let extension = input.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default();
    let data = std::fs::read(input).unwrap_or_else(|e| fail(&format!("{}: {e}", input.display())));
    let mut doc = Document::open(&extension, &data).unwrap_or_else(|e| fail(&format!("{}: {e}", input.display())));

    let mut settings: RenderSettings = doc.settings();
    for flag in &args[3..] {
        match flag.as_str() {
            "--8px" => settings.letter_spacing = false,
            "--9px" => settings.letter_spacing = true,
            "--ice" => settings.ice_colors = true,
            "--blink" => settings.ice_colors = false,
            other => fail(&format!("unknown flag {other}")),
        }
    }
    doc.set_settings(settings);

    let info = doc.info();
    let band = doc.render_rows(0, info.rows, true);
    let file = File::create(&args[2]).unwrap_or_else(|e| fail(&format!("{}: {e}", args[2])));
    let mut encoder = png::Encoder::new(BufWriter::new(file), band.width, band.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap_or_else(|e| fail(&e.to_string()));
    writer.write_image_data(&band.rgba).unwrap_or_else(|e| fail(&e.to_string()));
}

fn fail(message: &str) -> ! {
    eprintln!("render_png: {message}");
    std::process::exit(1);
}
