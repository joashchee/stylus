//! The editor's hot path without IPC (docs/roadmap.md, Phase 1b, "Hot path
//! benchmark"): what the core does for each move of a fast pencil drag.
//! Each move is the calls the Art workspace makes over Tauri IPC: `apply`
//! (the cells since the last move), `render_cells` (just those cells, both
//! blink frames when the art blinks) and `cell` (the status bar's cell under
//! the pointer). Set against the in-app numbers (the dev build's hot-path
//! log), the difference is what IPC and the webview cost.
//!
//! Usage: stroke_bench [moves]   (default 2000; build with --release)
//!
//! Prints, per canvas, the median, 95th percentile and worst time per move
//! for each call and in all.

use std::time::{Duration, Instant};

use stylus_core::{CellEdit, CellRect, Document, RenderSettings};

fn main() {
    let moves: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(2000);
    run("80×25", 80, 25, false, moves);
    run("160×1000", 160, 1000, false, moves);
    run("80×25, blink (two frames)", 80, 25, true, moves);
}

/// One pencil drag of `moves` moves over a new canvas: a zigzag across it,
/// one cell per move (a fast drag at 400% zoom moves about a cell per
/// pointer event), all one stroke, like the editor sends it.
fn run(label: &str, columns: i32, rows: i32, blink: bool, moves: usize) {
    let mut doc = Document::new_blank(columns, rows, !blink).unwrap();
    if blink {
        doc.set_settings(RenderSettings { ice_colors: false, ..doc.settings() });
    }
    let frames: &[bool] = if blink { &[true, false] } else { &[true] };
    let (mut apply, mut render, mut cell, mut total) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let rows_used = rows.min(25);
    for i in 0..moves {
        // Back and forth along each row, then down a row.
        let pass = i / columns as usize;
        let along = (i % columns as usize) as i32;
        let x = if pass % 2 == 0 { along } else { columns - 1 - along };
        let y = (pass as i32) % rows_used;
        let edit = CellEdit { x, y, code: Some(0xdb), fg: Some((i % 16) as u8), bg: Some(0) };

        let start = Instant::now();
        let dirty = doc.apply(1, &[edit]);
        let applied = Instant::now();
        let rect = dirty.unwrap_or(CellRect { x, y, width: 1, height: 1 });
        let mut bytes = 0;
        for &blink_on in frames {
            bytes += doc.render_cells(rect, blink_on).rgba.len();
        }
        let rendered = Instant::now();
        let under = doc.cell(x, y);
        let done = Instant::now();
        assert!(bytes > 0 && under.is_some());

        apply.push(applied - start);
        render.push(rendered - applied);
        cell.push(done - rendered);
        total.push(done - start);
    }
    println!("{label}, {moves} moves:");
    for (name, times) in [("apply", &mut apply), ("render_cells", &mut render), ("cell", &mut cell), ("per move", &mut total)] {
        times.sort();
        println!("  {name:<13} median {:>8}  p95 {:>8}  worst {:>8}", us(times[times.len() / 2]), us(times[times.len() * 95 / 100]), us(times[times.len() - 1]));
    }
}

fn us(d: Duration) -> String {
    format!("{:.1} µs", d.as_secs_f64() * 1e6)
}
