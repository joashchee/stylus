//! Stylus's desktop shell: Tauri commands over stylus-core, for file I/O
//! and OS integration. The art logic lives in stylus-core, never here, so
//! the web build can share it (Diskette's docs/stylus-notes.md,
//! "Architecture").

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use stylus_core::convert::{self, ConverterInfo};
use stylus_core::{Document, DocumentInfo, RenderSettings};
use tauri::ipc::Response;
use tauri::{AppHandle, Manager, State};

mod first_run;

/// Art the user has open, by id. The frontend holds only ids.
#[derive(Default)]
struct Library {
    next_id: u32,
    documents: HashMap<u32, Document>,
    /// The .ANS bytes of art Stylus made (Image to ANSI), by document id,
    /// for saving.
    made: HashMap<u32, Vec<u8>>,
    /// Images loaded for Image to ANSI, by id: name and pixels.
    images: HashMap<u32, (String, Arc<convert::RgbaImage>)>,
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
        library.made.remove(&id);
    }
}

/// Extensions the Image to ANSI panel takes.
#[tauri::command]
fn image_extensions() -> Vec<&'static str> {
    convert::IMAGE_EXTENSIONS.to_vec()
}

/// Every image-to-ANSI converter, in the panel's order.
#[tauri::command]
fn list_converters() -> Vec<&'static ConverterInfo> {
    convert::converters().iter().map(|c| c.info()).collect()
}

/// The libraries the converter ports carry code from, with license texts.
#[tauri::command]
fn library_notices() -> &'static [convert::LibraryNotice] {
    convert::LIBRARY_NOTICES
}

/// A converter's full license text, for About.
#[tauri::command]
fn converter_license(id: String) -> Result<&'static str, String> {
    convert::converter(&id).map(|c| c.info().license_text).ok_or_else(|| "No such converter".to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoadedImage {
    id: u32,
    name: String,
    width: u32,
    height: u32,
}

/// Reads and decodes an image for Image to ANSI. Only reads the file.
#[tauri::command]
async fn load_image(path: String, library: State<'_, Shared>) -> Result<LoadedImage, String> {
    let library = library.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let path = Path::new(&path);
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let extension = path.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default();
        let data = std::fs::read(path).map_err(|e| format!("Couldn't read {name}: {e}"))?;
        let image = convert::decode_image(&extension, &data).map_err(|e| format!("{name}: {e}"))?;
        let (width, height) = image.dimensions();
        let mut library = library.lock().map_err(|e| e.to_string())?;
        library.next_id += 1;
        let id = library.next_id;
        library.images.insert(id, (name.clone(), Arc::new(image)));
        Ok(LoadedImage { id, name, width, height })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn close_image(id: u32, library: State<'_, Shared>) {
    if let Ok(mut library) = library.lock() {
        library.images.remove(&id);
    }
}

/// Runs one converter on a loaded image and opens the .ANS it makes, as
/// art that can be rendered (render_band) and saved (save_made_art).
#[tauri::command]
async fn convert_image(image_id: u32, converter_id: String, library: State<'_, Shared>) -> Result<OpenedArt, String> {
    let library = library.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let (name, image) = library.lock().map_err(|e| e.to_string())?.images.get(&image_id).cloned().ok_or("That image isn't loaded")?;
        let converter = convert::converter(&converter_id).ok_or("No such converter")?;
        let stem = Path::new(&name).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let ans = convert::convert_to_ans(converter, &image, &stem, &today(), env!("CARGO_PKG_VERSION"));
        let document = Document::open("ans", &ans)?;
        let info = document.info();
        let mut library = library.lock().map_err(|e| e.to_string())?;
        library.next_id += 1;
        let id = library.next_id;
        library.documents.insert(id, document);
        library.made.insert(id, ans);
        Ok(OpenedArt { id, name: format!("{stem}-{}.ans", converter.info().id), info })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Writes art Stylus made to the path the user picked. `replace` is true
/// only when they already agreed to replace an existing file (the save
/// dialog asks); otherwise an existing file is left alone (CLAUDE.md
/// rule 4).
#[tauri::command]
fn save_made_art(id: u32, path: String, replace: bool, library: State<'_, Shared>) -> Result<(), String> {
    use std::io::Write;
    let library = library.lock().map_err(|e| e.to_string())?;
    let bytes = library.made.get(&id).ok_or("That art isn't open")?;
    let name = Path::new(&path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let mut file = if replace {
        std::fs::File::create(&path)
    } else {
        std::fs::OpenOptions::new().write(true).create_new(true).open(&path)
    }
    .map_err(|e| if e.kind() == std::io::ErrorKind::AlreadyExists { format!("{name} already exists") } else { format!("Couldn't save {name}: {e}") })?;
    file.write_all(bytes).map_err(|e| format!("Couldn't save {name}: {e}"))
}

/// Today as SAUCE's CCYYMMDD (UTC).
fn today() -> String {
    let days = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() / 86_400) as i64;
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}{m:02}{d:02}")
}

/// Writes the dev-only App Testing report (components/AppTesting.tsx) to
/// the path the user picked. Checklist status and notes only.
#[tauri::command]
fn export_app_testing_report(path: String, report: String) -> Result<(), String> {
    std::fs::write(&path, report).map_err(|e| e.to_string())
}

/// Creates the main window, once the first-run warning is accepted.
fn start(app: &AppHandle) {
    let config = &app.config().app.windows[0];
    tauri::WebviewWindowBuilder::from_config(app, config)
        .and_then(|builder| builder.build())
        .expect("failed to create the main window");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::new().level(log::LevelFilter::Info).build())
        .manage(Shared::default())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("failed to resolve app data dir");
            // The main window has `"create": false` in tauri.conf.json, so
            // nothing is drawn until the first-run warning is answered.
            if first_run::accepted(&app_data_dir) {
                start(app.handle());
            } else {
                first_run::ask(app.handle(), &app_data_dir, start);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            core_info,
            open_extensions,
            open_art,
            set_render_settings,
            render_band,
            close_art,
            image_extensions,
            list_converters,
            converter_license,
            library_notices,
            load_image,
            close_image,
            convert_image,
            save_made_art,
            export_app_testing_report
        ])
        .run(tauri::generate_context!())
        .expect("error while running Stylus");
}

#[cfg(test)]
mod tests {
    #[test]
    fn today_is_a_sauce_date() {
        let t = super::today();
        assert_eq!(t.len(), 8);
        assert!(t.starts_with("20"));
        assert!(t.chars().all(|c| c.is_ascii_digit()));
    }
}
