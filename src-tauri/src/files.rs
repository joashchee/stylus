//! Recent files and autosave, both in the app-data folder (docs/roadmap.md,
//! 1c "Files").
//!
//! - `recent.json`: the last files opened or saved, newest first.
//! - `Recovery/`: while a document has unsaved changes, the frontend asks
//!   for an autosave every few seconds; each writes a lossless snapshot
//!   (stylus-core's `recovery.rs`) as `<key>.stylus-recovery` with
//!   `<key>.json` beside it. Saving, discarding or closing the document
//!   removes them, so anything left at launch is from a crash, and the
//!   frontend offers it back. An autosave never touches the user's file
//!   (CLAUDE.md rule 4).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use stylus_core::Document;
use tauri::State;

use crate::{OpenedArt, Shared};

/// How many files File → Open Recent keeps.
const RECENT_MAX: usize = 10;
const RECENT_FILE: &str = "recent.json";
const RECOVERY_DIR: &str = "Recovery";
const SNAPSHOT_EXT: &str = "stylus-recovery";

/// The app-data folder. The lock keeps an autosave and a discard of the
/// same document from crossing, so a discarded autosave never comes back.
pub struct AppData {
    dir: PathBuf,
    io: Mutex<()>,
}

/// Shared with the blocking tasks that write autosaves.
pub type Data = Arc<AppData>;

impl AppData {
    pub fn new(dir: PathBuf) -> Data {
        Arc::new(AppData { dir, io: Mutex::new(()) })
    }

    fn recovery(&self) -> PathBuf {
        self.dir.join(RECOVERY_DIR)
    }
}

/// Writes a file whole or not at all: to a temporary name, then renamed.
fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut temp = path.as_os_str().to_owned();
    temp.push(".saving");
    let temp = PathBuf::from(temp);
    std::fs::write(&temp, bytes)?;
    std::fs::rename(&temp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&temp);
    })
}

// ---- Recent files ----

fn read_recent(dir: &Path) -> Vec<String> {
    std::fs::read(dir.join(RECENT_FILE)).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

fn write_recent(dir: &Path, list: &[String]) -> Result<(), String> {
    let json = serde_json::to_vec_pretty(list).map_err(|e| e.to_string())?;
    write_atomic(&dir.join(RECENT_FILE), &json).map_err(|e| format!("Couldn't update the recent files: {e}"))
}

/// `path` at the front of `list`, once, keeping at most `RECENT_MAX`.
fn remember(mut list: Vec<String>, path: &str) -> Vec<String> {
    list.retain(|p| p != path);
    list.insert(0, path.to_string());
    list.truncate(RECENT_MAX);
    list
}

/// The recent files, newest first.
#[tauri::command]
pub fn recent_files(data: State<'_, Data>) -> Vec<String> {
    let _io = data.io.lock();
    read_recent(&data.dir)
}

/// Puts a file just opened or saved at the top of the recent files.
#[tauri::command]
pub fn note_recent(path: String, data: State<'_, Data>) -> Result<Vec<String>, String> {
    let _io = data.io.lock();
    let list = remember(read_recent(&data.dir), &path);
    write_recent(&data.dir, &list)?;
    Ok(list)
}

/// Takes a file off the recent list (only if it's gone, with
/// `only_if_missing`), or clears the list when `path` is `None`.
#[tauri::command]
pub fn forget_recent(path: Option<String>, only_if_missing: bool, data: State<'_, Data>) -> Result<Vec<String>, String> {
    let _io = data.io.lock();
    let mut list = read_recent(&data.dir);
    match path {
        Some(path) if !only_if_missing || !Path::new(&path).exists() => list.retain(|p| *p != path),
        Some(_) => return Ok(list),
        None => list.clear(),
    }
    write_recent(&data.dir, &list)?;
    Ok(list)
}

// ---- Autosave and recovery ----

/// What a recovery file holds, for the offer at launch.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Recoverable {
    pub key: String,
    /// The art's name when it was autosaved.
    pub name: String,
    /// The file it came from, if it had one.
    pub path: Option<String>,
    /// Seconds since 1970.
    pub saved_at: u64,
}

/// A key the frontend made for a document: letters, digits and dashes only,
/// so it can't name a file outside the recovery folder.
fn check_key(key: &str) -> Result<(), String> {
    if (1..=64).contains(&key.len()) && key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        Ok(())
    } else {
        Err("That isn't a recovery key".to_string())
    }
}

fn snapshot_path(dir: &Path, key: &str) -> PathBuf {
    dir.join(format!("{key}.{SNAPSHOT_EXT}"))
}

fn meta_path(dir: &Path, key: &str) -> PathBuf {
    dir.join(format!("{key}.json"))
}

fn remove_recovery(dir: &Path, key: &str) {
    let _ = std::fs::remove_file(meta_path(dir, key));
    let _ = std::fs::remove_file(snapshot_path(dir, key));
}

/// Every recovery file left, newest first. A half-written pair (the app
/// stopped between the two files) is cleared away.
fn list_recoverable(dir: &Path) -> Vec<Recoverable> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut found: Vec<Recoverable> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let meta: Option<Recoverable> = std::fs::read(&path).ok().and_then(|b| serde_json::from_slice(&b).ok());
        match meta {
            Some(meta) if check_key(&meta.key).is_ok() && snapshot_path(dir, &meta.key).exists() => found.push(meta),
            _ => {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
    found.sort_by(|a, b| b.saved_at.cmp(&a.saved_at));
    found
}

/// Writes an autosave of document `id` under `key` if it has unsaved
/// changes the last autosave didn't have. A document with nothing unsaved
/// has its autosave removed. Returns whether it wrote.
#[tauri::command]
pub async fn autosave(id: u32, key: String, name: String, path: Option<String>, library: State<'_, Shared>, data: State<'_, Data>) -> Result<bool, String> {
    check_key(&key)?;
    let library = library.inner().clone();
    let data = data.inner().clone();
    let dir = data.recovery();
    tauri::async_runtime::spawn_blocking(move || {
        let _io = data.io.lock().map_err(|e| e.to_string())?;
        let snapshot = {
            let mut library = library.lock().map_err(|e| e.to_string())?;
            let document = library.documents.get(&id).ok_or("That art isn't open")?;
            if !document.is_edited() {
                library.autosaved.remove(&key);
                drop(library);
                remove_recovery(&dir, &key);
                return Ok(false);
            }
            let state = (document.version(), document.settings());
            if library.autosaved.get(&key) == Some(&state) {
                return Ok(false);
            }
            let snapshot = document.recovery_snapshot()?;
            library.autosaved.insert(key.clone(), state);
            snapshot
        };
        let saved_at = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
        let meta = serde_json::to_vec_pretty(&Recoverable { key: key.clone(), name, path, saved_at }).map_err(|e| e.to_string())?;
        // The art first: a meta file always has its art beside it.
        write_atomic(&snapshot_path(&dir, &key), &snapshot)
            .and_then(|()| write_atomic(&meta_path(&dir, &key), &meta))
            .map_err(|e| format!("Couldn't autosave: {e}"))?;
        Ok(true)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Removes a document's autosave: it was saved, discarded or closed.
#[tauri::command]
pub fn discard_recovery(key: String, library: State<'_, Shared>, data: State<'_, Data>) -> Result<(), String> {
    check_key(&key)?;
    let _io = data.io.lock().map_err(|e| e.to_string())?;
    if let Ok(mut library) = library.lock() {
        library.autosaved.remove(&key);
    }
    remove_recovery(&data.recovery(), &key);
    Ok(())
}

/// Autosaves left behind when Stylus last stopped without saving.
#[tauri::command]
pub fn recoverable(data: State<'_, Data>) -> Vec<Recoverable> {
    let _io = data.io.lock();
    list_recoverable(&data.recovery())
}

/// Opens an autosave as a document with unsaved changes. Its files stay
/// until the document is saved or discarded, and later autosaves of it go
/// to the same key.
#[tauri::command]
pub fn recover(key: String, library: State<'_, Shared>, data: State<'_, Data>) -> Result<OpenedArt, String> {
    check_key(&key)?;
    let dir = data.recovery();
    let meta: Recoverable = std::fs::read(meta_path(&dir, &key))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .ok_or("That recovered art is gone")?;
    let bytes = std::fs::read(snapshot_path(&dir, &key)).map_err(|e| format!("Couldn't read the recovered art: {e}"))?;
    let document = Document::from_recovery(&bytes)?;
    let info = document.info();
    let mut library = library.lock().map_err(|e| e.to_string())?;
    library.next_id += 1;
    let id = library.next_id;
    library.autosaved.insert(key, (document.version(), document.settings()));
    library.documents.insert(id, document);
    Ok(OpenedArt { id, name: meta.name, info })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("stylus-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn recent_files_are_newest_first_once_each_and_capped() {
        let mut list = Vec::new();
        for i in 0..12 {
            list = remember(list, &format!("/art/{i}.ans"));
        }
        assert_eq!(list.len(), RECENT_MAX);
        assert_eq!(list[0], "/art/11.ans");
        list = remember(list, "/art/5.ans");
        assert_eq!(list[0], "/art/5.ans");
        assert_eq!(list.iter().filter(|p| *p == "/art/5.ans").count(), 1);

        let dir = temp_dir("recent");
        assert!(read_recent(&dir).is_empty());
        write_recent(&dir, &list).unwrap();
        assert_eq!(read_recent(&dir), list);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn keys_cannot_leave_the_folder() {
        assert!(check_key("m1abc-42").is_ok());
        for bad in ["", "../recent", "a/b", "a.b", &"x".repeat(65)] {
            assert!(check_key(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn only_whole_autosaves_are_offered_newest_first() {
        let dir = temp_dir("recovery");
        let put = |key: &str, saved_at: u64, art: bool| {
            if art {
                write_atomic(&snapshot_path(&dir, key), b"art").unwrap();
            }
            let meta = Recoverable { key: key.into(), name: format!("{key}.ans"), path: None, saved_at };
            write_atomic(&meta_path(&dir, key), &serde_json::to_vec(&meta).unwrap()).unwrap();
        };
        put("old", 100, true);
        put("new", 200, true);
        put("half", 300, false);
        std::fs::write(dir.join("junk.json"), b"not json").unwrap();
        let keys: Vec<String> = list_recoverable(&dir).into_iter().map(|r| r.key).collect();
        assert_eq!(keys, ["new", "old"]);
        assert!(!meta_path(&dir, "half").exists() && !dir.join("junk.json").exists(), "strays cleared");
        remove_recovery(&dir, "new");
        assert_eq!(list_recoverable(&dir).len(), 1);
        assert!(list_recoverable(&dir.join("missing")).is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
