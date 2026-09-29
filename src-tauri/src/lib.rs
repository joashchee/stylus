//! Stylus's desktop shell: Tauri commands over stylus-core, for file I/O
//! and OS integration. The art logic lives in stylus-core, never here, so
//! the web build can share it (Diskette's docs/stylus-notes.md,
//! "Architecture").

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use stylus_core::{Document, DocumentInfo, RenderSettings};
use tauri::ipc::Response;
use tauri::State;

/// Art the user has open, by id. The frontend holds only ids.
#[derive(Default)]
struct Library {
    next_id: u32,
    documents: HashMap<u32, Document>,
}

type Shared = Arc<Mutex<Library>>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenedArt {
    id: u32,
    /// The file name, for the title and the recent list.
    name: String,
    info: DocumentInfo,
}

/// stylus-core's version and the icy_tools revision it's pinned to.
#[tauri::command]
fn core_info() -> stylus_core::CoreInfo {
    stylus_core::core_info()
}

/// Extensions the viewer opens, for the file dialog and drag and drop.
#[tauri::command]
fn open_extensions() -> Vec<&'static str> {
    stylus_core::OPEN_EXTENSIONS.to_vec()
}

/// Opens a file for viewing. Only reads it: opening never changes the file
/// (CLAUDE.md rule 4).
#[tauri::command]
async fn open_art(path: String, library: State<'_, Shared>) -> Result<OpenedArt, String> {
    let library = library.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let path = Path::new(&path);
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let extension = path.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default();
        let data = std::fs::read(path).map_err(|e| format!("Couldn't read {name}: {e}"))?;
        let document = Document::open(&extension, &data).map_err(|e| format!("Couldn't open {name}: {e}"))?;
        let info = document.info();
        let mut library = library.lock().map_err(|e| e.to_string())?;
        library.next_id += 1;
        let id = library.next_id;
        library.documents.insert(id, document);
        Ok(OpenedArt { id, name, info })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn set_render_settings(id: u32, settings: RenderSettings, library: State<'_, Shared>) -> Result<DocumentInfo, String> {
    let library = library.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut library = library.lock().map_err(|e| e.to_string())?;
        let document = library.documents.get_mut(&id).ok_or("That art isn't open")?;
        document.set_settings(settings);
        Ok(document.info())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Renders a band of rows as raw bytes: width and height (u32, little
/// endian), then RGBA. Raw so a band doesn't go through JSON.
#[tauri::command]
async fn render_band(id: u32, first_row: i32, row_count: i32, blink_on: bool, library: State<'_, Shared>) -> Result<Response, String> {
    let library = library.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let library = library.lock().map_err(|e| e.to_string())?;
        let document = library.documents.get(&id).ok_or("That art isn't open")?;
        let band = document.render_rows(first_row, row_count, blink_on);
        let mut bytes = Vec::with_capacity(8 + band.rgba.len());
        bytes.extend_from_slice(&band.width.to_le_bytes());
        bytes.extend_from_slice(&band.height.to_le_bytes());
        bytes.extend_from_slice(&band.rgba);
        Ok(Response::new(bytes))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn close_art(id: u32, library: State<'_, Shared>) {
    if let Ok(mut library) = library.lock() {
        library.documents.remove(&id);
    }
}

/// Writes the dev-only App Testing report (components/AppTesting.tsx) to
/// the path the user picked. Checklist status and notes only.
#[tauri::command]
fn export_app_testing_report(path: String, report: String) -> Result<(), String> {
    std::fs::write(&path, report).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::new().level(log::LevelFilter::Info).build())
        .manage(Shared::default())
        .invoke_handler(tauri::generate_handler![
            core_info,
            open_extensions,
            open_art,
            set_render_settings,
            render_band,
            close_art,
            export_app_testing_report
        ])
        .run(tauri::generate_context!())
        .expect("error while running Stylus");
}
