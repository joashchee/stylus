//! Compares two PNGs pixel by pixel, ignoring alpha. For
//! scripts/compare-ansilove.sh.
//!
//! Usage: png_diff <a.png> <b.png>
//! Prints "same", or what differs: the sizes, or how many pixels differ in
//! the area both cover. Exits 0 when the same, 1 when not, 2 on errors.

use std::fs::File;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: png_diff <a.png> <b.png>");
        std::process::exit(2);
    }
    let (a, aw, ah) = load(&args[1]);
    let (b, bw, bh) = load(&args[2]);
    let (w, h) = (aw.min(bw), ah.min(bh));
    let mut differing = 0u64;
    for y in 0..h {
        for x in 0..w {
            if a[(y * aw + x) as usize] != b[(y * bw + x) as usize] {
                differing += 1;
            }
        }
    }
    if differing == 0 && (aw, ah) == (bw, bh) {
        println!("same");
        return;
    }
    let size = if (aw, ah) == (bw, bh) { format!("{aw}x{ah}") } else { format!("sizes differ: {aw}x{ah} vs {bw}x{bh}") };
    println!("{size}, {differing} of {} shared pixels differ", u64::from(w) * u64::from(h));
    std::process::exit(1);
}

/// Pixels as RGB triples packed in u32, with the width and height.
fn load(path: &str) -> (Vec<u32>, u32, u32) {
    let file = File::open(path).unwrap_or_else(|e| fail(&format!("{path}: {e}")));
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::normalize_to_color8() | png::Transformations::ALPHA);
    let mut reader = decoder.read_info().unwrap_or_else(|e| fail(&format!("{path}: {e}")));
    let mut buf = vec![0; reader.output_buffer_size().unwrap_or(0)];
    let frame = reader.next_frame(&mut buf).unwrap_or_else(|e| fail(&format!("{path}: {e}")));
    let channels = frame.color_type.samples();
    let pixels = buf[..frame.buffer_size()]
        .chunks(channels)
        .map(|p| match channels {
            1 | 2 => u32::from_be_bytes([0, p[0], p[0], p[0]]),
            _ => u32::from_be_bytes([0, p[0], p[1], p[2]]),
        })
        .collect();
    (pixels, frame.width, frame.height)
}

fn fail(message: &str) -> ! {
    eprintln!("png_diff: {message}");
    std::process::exit(2);
}
