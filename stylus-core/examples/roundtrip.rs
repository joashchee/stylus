//! Saving tested on real art (docs/roadmap.md, Phase 1c): opens every art
//! file under a folder, saves it again (in its own format when Stylus saves
//! that, and as .ANS and .XB), reopens what was saved and compares the
//! rendered pixels with the original's. For scripts/roundtrip-corpus.sh.
//!
//! Usage: roundtrip <corpus-folder>
//!
//! Prints one line per file: each target format and "same", "N of M pixels
//! differ", "can't save: …" (a blocking loss, e.g. BIN wider than 255), or
//! the error. A save that loses something by design says so after "lossy:",
//! and differing pixels there are expected.

use std::path::{Path, PathBuf};

use stylus_core::{Document, SauceFields, OPEN_EXTENSIONS, SAVE_FORMATS};

fn main() {
    let Some(root) = std::env::args().nth(1) else {
        eprintln!("usage: roundtrip <corpus-folder>");
        std::process::exit(2);
    };
    let mut files = Vec::new();
    collect(Path::new(&root), &mut files);
    files.sort();
    let (mut same, mut lossy, mut differ, mut failed) = (0, 0, 0, 0);
    for file in &files {
        let extension = file.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
        let data = match std::fs::read(file) {
            Ok(d) => d,
            Err(e) => {
                println!("{}: can't read: {e}", file.display());
                failed += 1;
                continue;
            }
        };
        let doc = match Document::open(&extension, &data) {
            Ok(d) => d,
            Err(e) => {
                println!("{}: can't open: {e}", file.display());
                failed += 1;
                continue;
            }
        };
        let info = doc.info();
        let original = doc.render_rows(0, info.rows, true);
        let sauce = info.sauce.as_ref().map_or_else(SauceFields::default, |s| SauceFields {
            title: s.title.clone(),
            author: s.author.clone(),
            group: s.group.clone(),
            date: s.date.replace('/', ""),
            comments: s.comments.clone(),
        });
        let mut targets: Vec<&str> = Vec::new();
        if let Some(own) = SAVE_FORMATS.iter().find(|f| f.extension == extension) {
            targets.push(own.extension);
        }
        for t in ["ans", "xb"] {
            if !targets.contains(&t) {
                targets.push(t);
            }
        }
        let mut results = Vec::new();
        for target in targets {
            let losses = match doc.save_losses(target) {
                Ok(l) => l,
                Err(e) => {
                    results.push(format!("{target}: {e}"));
                    failed += 1;
                    continue;
                }
            };
            if let Some(block) = losses.iter().find(|l| l.blocking) {
                results.push(format!("{target}: can't save: {}", block.message));
                continue;
            }
            let outcome = doc.save(target, &sauce).and_then(|bytes| Document::open(target, &bytes)).map(|mut reopened| {
                reopened.set_settings(doc.settings());
                let r = reopened.info();
                let band = reopened.render_rows(0, r.rows, true);
                (band, r.columns, r.rows)
            });
            match outcome {
                Err(e) => {
                    results.push(format!("{target}: failed: {e}"));
                    failed += 1;
                }
                Ok((band, columns, rows)) => {
                    let note = if losses.is_empty() {
                        String::new()
                    } else {
                        format!(" (lossy: {})", losses.iter().map(|l| l.message.as_str()).collect::<Vec<_>>().join("; "))
                    };
                    if band.width != original.width || band.height != original.height {
                        results.push(format!("{target}: size {columns}x{rows}, was {}x{}{note}", info.columns, info.rows));
                        if losses.is_empty() { differ += 1 } else { lossy += 1 }
                    } else {
                        let n = band.rgba.chunks(4).zip(original.rgba.chunks(4)).filter(|(a, b)| a != b).count();
                        if n == 0 {
                            results.push(format!("{target}: same"));
                            same += 1;
                        } else {
                            results.push(format!("{target}: {n} of {} pixels differ{note}", band.rgba.len() / 4));
                            if losses.is_empty() { differ += 1 } else { lossy += 1 }
                        }
                    }
                }
            }
        }
        println!("{}: {}", file.display(), results.join(", "));
    }
    println!("\n{} files; saves: {same} same, {lossy} differ with a stated loss, {differ} differ unexplained, {failed} failed", files.len());
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|e| OPEN_EXTENSIONS.contains(&e.to_string_lossy().to_ascii_lowercase().as_str())) {
            out.push(path);
        }
    }
}
