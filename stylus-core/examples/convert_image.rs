//! Runs every Image to ANSI converter on an image and writes each one's
//! .ANS and a PNG of it as the viewer draws it, for checking the ports
//! against the originals by eye.
//!
//! Usage: convert_image <image> <output folder> [converter id]

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::Instant;

use stylus_core::convert::{convert_to_ans, converters, decode_image, Color, COLUMNS};
use stylus_core::Document;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: convert_image <image> <output folder> [converter id]");
        std::process::exit(2);
    }
    let input = Path::new(&args[1]);
    let out = Path::new(&args[2]);
    std::fs::create_dir_all(out).expect("output folder");
    let extension = input.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default();
    let data = std::fs::read(input).expect("image");
    let image = decode_image(&extension, &data).expect("decode");
    for c in converters() {
        let id = c.info().id;
        if args.get(3).is_some_and(|only| only != id) {
            continue;
        }
        // A text dump of the cells, one row per line, for comparing a port
        // with its original: "ch:fg/bg" with ch in hex, colors as A<n> (ANSI,
        // SGR order), X<n> (xterm), R<r>,<g>,<b> or D (default).
        let grid = c.convert(&image, COLUMNS);
        let color = |c: Color| match c {
            Color::Default => "D".to_string(),
            Color::Ansi(i) => format!("A{i}"),
            Color::Xterm(i) => format!("X{i}"),
            Color::Rgb(r, g, b) => format!("R{r},{g},{b}"),
        };
        let dump: Vec<String> = (0..grid.height)
            .map(|y| (0..grid.width).map(|x| grid.get(x, y)).map(|cell| format!("{:02x}:{}/{}", cell.ch, color(cell.fg), color(cell.bg))).collect::<Vec<_>>().join(" "))
            .collect();
        std::fs::write(out.join(format!("{id}.cells")), dump.join("\n") + "\n").expect("write cells");
        let start = Instant::now();
        let ans = convert_to_ans(*c, &image, "test", "20260929", "dev");
        let took = start.elapsed();
        std::fs::write(out.join(format!("{id}.ans")), &ans).expect("write ans");
        let doc = Document::open("ans", &ans).expect("open");
        let info = doc.info();
        let band = doc.render_rows(0, info.rows, true);
        let file = File::create(out.join(format!("{id}.png"))).expect("png");
        let mut encoder = png::Encoder::new(BufWriter::new(file), band.width, band.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header().unwrap().write_image_data(&band.rgba).unwrap();
        println!("{id:28} {}x{} in {:?}", info.columns, info.rows, took);
    }
}
