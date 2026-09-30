//! Stylus's desktop shell: Tauri commands over stylus-core, for file I/O
//! and OS integration. The art logic lives in stylus-core, never here, so
//! the web build can share it (Diskette's docs/stylus-notes.md,
//! "Architecture").

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use stylus_core::contrast::ContrastProblem;
use stylus_core::convert::{self, ConverterInfo};
use stylus_core::{CellEdit, CellInfo, CellRect, Clip, Document, DocumentInfo, Pen, PngExport, PngOptions, PngSize, Point, RenderSettings, SaveFormat, SaveLoss, SauceFields, SelectionOp, Shape};
use tauri::ipc::Response;
use tauri::{AppHandle, Manager, State};

mod files;
mod first_run;
mod native_menu;

/// Art the user has open, by id. The frontend holds only ids.
#[derive(Default)]
struct Library {
    next_id: u32,
    /// Numbers "untitled-N" documents.
    untitled: u32,
    documents: HashMap<u32, Document>,
    /// The .ANS bytes of art Stylus made (Image to ANSI), by document id,
    /// for saving.
    made: HashMap<u32, Vec<u8>>,
    /// Images loaded for Image to ANSI, by id: name and pixels.
    images: HashMap<u32, (String, Arc<convert::RgbaImage>)>,
    /// Cells copied or cut, for pasting into any open document.
    clip: Option<Clip>,
    /// What each autosave last wrote (files.rs), by recovery key: the
    /// document's version and render settings, so an unchanged one is skipped.
    autosaved: HashMap<String, (u64, RenderSettings)>,
    /// PNG exports under way, by export id.
    exports: HashMap<u32, PngJob>,
}

/// A PNG export being written to a temporary file beside `target`, moved
/// over it once the last row is in, so a failed or cancelled export leaves
/// nothing half-written.
struct PngJob {
    document: u32,
    export: PngExport<std::io::BufWriter<std::fs::File>>,
    /// The same file, kept to sync it once the PNG is finished.
    file: std::fs::File,
    temp: std::path::PathBuf,
    target: std::path::PathBuf,
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

/// What an edit, undo or redo changed: the cells to redraw, the history
/// state for the Edit menu, and the new info when the size changed.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EditResult {
    dirty: Option<CellRect>,
    can_undo: bool,
    can_redo: bool,
    edited: bool,
    /// Set when the canvas size changed (an undone or redone resize).
    info: Option<DocumentInfo>,
}

fn edit_result(document: &Document, dirty: Option<CellRect>, columns: i32, rows: i32) -> EditResult {
    let resized = document.columns() != columns || document.rows() != rows;
    EditResult {
        dirty,
        can_undo: document.can_undo(),
        can_redo: document.can_redo(),
        edited: document.is_edited(),
        info: resized.then(|| document.info()),
    }
}

/// A new, empty document.
#[tauri::command]
fn new_art(columns: i32, rows: i32, ice_colors: bool, library: State<'_, Shared>) -> Result<OpenedArt, String> {
    let document = Document::new_blank(columns, rows, ice_colors)?;
    let info = document.info();
    let mut library = library.lock().map_err(|e| e.to_string())?;
    library.next_id += 1;
    library.untitled += 1;
    let id = library.next_id;
    let name = format!("untitled-{}.ans", library.untitled);
    library.documents.insert(id, document);
    Ok(OpenedArt { id, name, info })
}

#[tauri::command]
fn document_info(id: u32, library: State<'_, Shared>) -> Result<DocumentInfo, String> {
    let library = library.lock().map_err(|e| e.to_string())?;
    Ok(library.documents.get(&id).ok_or("That art isn't open")?.info())
}

/// Applies cell edits as part of one stroke (edits with the same stroke id
/// undo together).
#[tauri::command]
fn apply_edits(id: u32, stroke: u32, edits: Vec<CellEdit>, library: State<'_, Shared>) -> Result<EditResult, String> {
    let mut library = library.lock().map_err(|e| e.to_string())?;
    let document = library.documents.get_mut(&id).ok_or("That art isn't open")?;
    let (columns, rows) = (document.columns(), document.rows());
    let dirty = document.apply(stroke, &edits);
    Ok(edit_result(document, dirty, columns, rows))
}

#[tauri::command]
fn undo(id: u32, library: State<'_, Shared>) -> Result<EditResult, String> {
    let mut library = library.lock().map_err(|e| e.to_string())?;
    let document = library.documents.get_mut(&id).ok_or("That art isn't open")?;
    let (columns, rows) = (document.columns(), document.rows());
    let dirty = document.undo();
    Ok(edit_result(document, dirty, columns, rows))
}

#[tauri::command]
fn redo(id: u32, library: State<'_, Shared>) -> Result<EditResult, String> {
    let mut library = library.lock().map_err(|e| e.to_string())?;
    let document = library.documents.get_mut(&id).ok_or("That art isn't open")?;
    let (columns, rows) = (document.columns(), document.rows());
    let dirty = document.redo();
    Ok(edit_result(document, dirty, columns, rows))
}

/// Runs `change` on an open document and reports what it changed.
fn edit_with(library: &State<'_, Shared>, id: u32, change: impl FnOnce(&mut Document) -> Option<CellRect>) -> Result<EditResult, String> {
    let mut library = library.lock().map_err(|e| e.to_string())?;
    let document = library.documents.get_mut(&id).ok_or("That art isn't open")?;
    let (columns, rows) = (document.columns(), document.rows());
    let dirty = change(document);
    Ok(edit_result(document, dirty, columns, rows))
}

/// Draws a line, rectangle or box as stroke `stroke`, replacing what the
/// same stroke drew before (called at each move of a drag).
#[tauri::command]
fn draw_shape(id: u32, stroke: u32, shape: Shape, pen: Pen, library: State<'_, Shared>) -> Result<EditResult, String> {
    edit_with(&library, id, |d| d.draw_shape(stroke, shape, pen))
}

#[tauri::command]
fn flood_fill(id: u32, stroke: u32, at: Point, pen: Pen, library: State<'_, Shared>) -> Result<EditResult, String> {
    edit_with(&library, id, |d| d.flood_fill(stroke, at, pen))
}

/// The half-block brush from `from` to `to`, in half-row pixels.
#[tauri::command]
fn half_block(id: u32, stroke: u32, from: Point, to: Point, color: u8, library: State<'_, Shared>) -> Result<EditResult, String> {
    edit_with(&library, id, |d| d.half_block(stroke, from, to, color))
}

/// Clears, fills, flips or moves the selected cells.
#[tauri::command]
fn selection_op(id: u32, stroke: u32, rect: CellRect, op: SelectionOp, library: State<'_, Shared>) -> Result<EditResult, String> {
    edit_with(&library, id, |d| d.selection(stroke, rect, op))
}

/// The size of what's on Stylus's clipboard.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClipSize {
    width: i32,
    height: i32,
}

/// Copies the cells in `rect` to Stylus's clipboard.
#[tauri::command]
fn copy_cells(id: u32, rect: CellRect, library: State<'_, Shared>) -> Result<Option<ClipSize>, String> {
    let mut library = library.lock().map_err(|e| e.to_string())?;
    let clip = library.documents.get(&id).ok_or("That art isn't open")?.copy(rect);
    let size = clip.as_ref().map(|c| ClipSize { width: c.width, height: c.height });
    if clip.is_some() {
        library.clip = clip;
    }
    Ok(size)
}

/// Pastes Stylus's clipboard with its top left at `at`, and says where it went.
#[tauri::command]
fn paste_cells(id: u32, stroke: u32, at: Point, transparent: bool, library: State<'_, Shared>) -> Result<(EditResult, Option<CellRect>), String> {
    let mut library = library.lock().map_err(|e| e.to_string())?;
    let library = &mut *library;
    let Some(clip) = library.clip.as_ref() else {
        return Ok((edit_result_of(library.documents.get(&id).ok_or("That art isn't open")?), None));
    };
    let document = library.documents.get_mut(&id).ok_or("That art isn't open")?;
    let (columns, rows) = (document.columns(), document.rows());
    let dirty = document.paste(stroke, clip, at, transparent);
    let placed = CellRect { x: at.x, y: at.y, width: clip.width, height: clip.height };
    Ok((edit_result(document, dirty, columns, rows), Some(placed)))
}

fn edit_result_of(document: &Document) -> EditResult {
    edit_result(document, None, document.columns(), document.rows())
}

#[tauri::command]
fn resize_art(id: u32, columns: i32, rows: i32, library: State<'_, Shared>) -> Result<DocumentInfo, String> {
    let mut library = library.lock().map_err(|e| e.to_string())?;
    let document = library.documents.get_mut(&id).ok_or("That art isn't open")?;
    document.resize(columns, rows)?;
    Ok(document.info())
}

#[tauri::command]
fn cell_at(id: u32, x: i32, y: i32, library: State<'_, Shared>) -> Result<Option<CellInfo>, String> {
    let library = library.lock().map_err(|e| e.to_string())?;
    Ok(library.documents.get(&id).ok_or("That art isn't open")?.cell(x, y))
}

/// The contrast lint: every cell short of the contrast its role needs.
#[tauri::command]
fn contrast_problems(id: u32, library: State<'_, Shared>) -> Result<Vec<ContrastProblem>, String> {
    let library = library.lock().map_err(|e| e.to_string())?;
    Ok(library.documents.get(&id).ok_or("That art isn't open")?.contrast_problems())
}

/// Renders a rectangle of cells as raw bytes, like `render_band`.
#[tauri::command]
fn render_cells(id: u32, rect: CellRect, blink_on: bool, library: State<'_, Shared>) -> Result<Response, String> {
    let library = library.lock().map_err(|e| e.to_string())?;
    let document = library.documents.get(&id).ok_or("That art isn't open")?;
    let band = document.render_cells(rect, blink_on);
    let mut bytes = Vec::with_capacity(8 + band.rgba.len());
    bytes.extend_from_slice(&band.width.to_le_bytes());
    bytes.extend_from_slice(&band.height.to_le_bytes());
    bytes.extend_from_slice(&band.rgba);
    Ok(Response::new(bytes))
}

/// Formats Save As offers.
#[tauri::command]
fn save_formats() -> &'static [SaveFormat] {
    stylus_core::SAVE_FORMATS
}

/// What saving in a format would lose, for the Save As warning.
#[tauri::command]
fn save_losses(id: u32, extension: String, library: State<'_, Shared>) -> Result<Vec<SaveLoss>, String> {
    let library = library.lock().map_err(|e| e.to_string())?;
    library.documents.get(&id).ok_or("That art isn't open")?.save_losses(&extension)
}

/// Saves a document to `path`, in the format its extension names, with one
/// SAUCE record from `sauce`. `replace` is true only when the user agreed
/// to replace the file (the save dialog asked, or it's the document's own
/// file and they chose Save). A replaced file is written next to it first
/// and moved over it, so a failed save never leaves it half-written.
#[tauri::command]
async fn save_art(id: u32, path: String, mut sauce: SauceFields, replace: bool, library: State<'_, Shared>) -> Result<DocumentInfo, String> {
    let library = library.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let target = Path::new(&path);
        let name = target.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let extension = target.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default();
        if sauce.date.is_empty() {
            sauce.date = today();
        }
        let bytes = {
            let library = library.lock().map_err(|e| e.to_string())?;
            library.documents.get(&id).ok_or("That art isn't open")?.save(&extension, &sauce)?
        };
        write_file(target, &bytes, replace).map_err(|e| format!("Couldn't save {name}: {e}"))?;
        let mut library = library.lock().map_err(|e| e.to_string())?;
        let document = library.documents.get_mut(&id).ok_or("That art isn't open")?;
        document.mark_saved();
        Ok(document.info())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Writes `bytes` to `path`: a new file only, unless `replace`, and then
/// through a temporary file beside it renamed into place.
fn write_file(path: &Path, bytes: &[u8], replace: bool) -> std::io::Result<()> {
    use std::io::Write;
    if !replace {
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                std::io::Error::new(e.kind(), "it already exists")
            } else {
                e
            }
        })?;
        return file.write_all(bytes);
    }
    let dir = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let temp = dir.join(format!(".{stem}.stylus-saving"));
    let result = (|| {
        let mut file = std::fs::File::create(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        std::fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

/// The size a PNG export would be, for the export dialog.
#[tauri::command]
fn png_size(id: u32, options: PngOptions, library: State<'_, Shared>) -> Result<PngSize, String> {
    let mut library = library.lock().map_err(|e| e.to_string())?;
    Ok(library.documents.get_mut(&id).ok_or("That art isn't open")?.png_size(options))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PngStarted {
    export: u32,
    /// Art rows to write, for the progress bar.
    rows: i32,
}

/// Starts exporting a document to a PNG at `path`. The frontend then calls
/// `png_export_rows` until every row is written. The system save dialog has
/// already asked before replacing a file.
#[tauri::command]
fn begin_png_export(id: u32, path: String, options: PngOptions, library: State<'_, Shared>) -> Result<PngStarted, String> {
    let target = std::path::PathBuf::from(&path);
    let name = target.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let temp = target.with_file_name(format!(".{name}.stylus-saving"));
    let mut library = library.lock().map_err(|e| e.to_string())?;
    let document = library.documents.get_mut(&id).ok_or("That art isn't open")?;
    let file = std::fs::File::create(&temp).map_err(|e| e.to_string())?;
    let started = file
        .try_clone()
        .map_err(|e| e.to_string())
        .and_then(|clone| PngExport::begin(document, options, std::io::BufWriter::new(clone)));
    let export = match started {
        Ok(export) => export,
        Err(e) => {
            let _ = std::fs::remove_file(&temp);
            return Err(e);
        }
    };
    let rows = export.rows();
    library.next_id += 1;
    let export_id = library.next_id;
    library.exports.insert(export_id, PngJob { document: id, export, file, temp, target });
    Ok(PngStarted { export: export_id, rows })
}

/// Writes the next `rows` art rows of a PNG export and returns how many
/// are done. After the last, the PNG is finished and moved into place.
#[tauri::command]
async fn png_export_rows(export: u32, rows: i32, library: State<'_, Shared>) -> Result<i32, String> {
    let library = library.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = library.lock().map_err(|e| e.to_string())?;
        let library = &mut *guard;
        let job = library.exports.get_mut(&export).ok_or("That export isn't running")?;
        let written = match library.documents.get_mut(&job.document) {
            Some(document) => job.export.write_rows(document, rows),
            None => Err("The art was closed".to_string()),
        };
        let done = match written {
            Ok(done) => done,
            Err(e) => {
                cancel_job(library.exports.remove(&export));
                return Err(e);
            }
        };
        if library.exports.get(&export).is_some_and(|job| job.export.is_done()) {
            let job = library.exports.remove(&export).expect("checked above");
            let (file, temp, target) = (job.file, job.temp, job.target);
            let finished = job
                .export
                .finish()
                .and_then(|()| file.sync_all().map_err(|e| e.to_string()))
                .and_then(|()| std::fs::rename(&temp, &target).map_err(|e| e.to_string()));
            if let Err(e) = finished {
                let _ = std::fs::remove_file(&temp);
                return Err(e);
            }
        }
        Ok(done)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Stops a PNG export and removes its temporary file.
#[tauri::command]
fn cancel_png_export(export: u32, library: State<'_, Shared>) {
    if let Ok(mut library) = library.lock() {
        cancel_job(library.exports.remove(&export));
    }
}

fn cancel_job(job: Option<PngJob>) {
    if let Some(job) = job {
        let temp = job.temp;
        drop(job.export);
        drop(job.file);
        let _ = std::fs::remove_file(temp);
    }
}

/// Fonts a text file can be shown in (Amiga ASCII).
#[tauri::command]
fn text_fonts() -> &'static [&'static str] {
    stylus_core::TEXT_FONTS
}

#[tauri::command]
fn set_text_font(id: u32, name: String, library: State<'_, Shared>) -> Result<DocumentInfo, String> {
    let mut library = library.lock().map_err(|e| e.to_string())?;
    let document = library.documents.get_mut(&id).ok_or("That art isn't open")?;
    document.set_text_font(&name)?;
    Ok(document.info())
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
        .manage(native_menu::NativeMenu::default())
        .on_menu_event(|app, event| native_menu::on_menu_event(app, event.id().as_ref()))
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("failed to resolve app data dir");
            app.manage(files::AppData::new(app_data_dir.clone()));
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
            new_art,
            document_info,
            apply_edits,
            undo,
            redo,
            resize_art,
            cell_at,
            render_cells,
            draw_shape,
            flood_fill,
            half_block,
            selection_op,
            copy_cells,
            paste_cells,
            save_formats,
            save_losses,
            contrast_problems,
            save_art,
            png_size,
            begin_png_export,
            png_export_rows,
            cancel_png_export,
            text_fonts,
            set_text_font,
            image_extensions,
            list_converters,
            converter_license,
            library_notices,
            load_image,
            close_image,
            convert_image,
            save_made_art,
            export_app_testing_report,
            files::recent_files,
            files::note_recent,
            files::forget_recent,
            files::autosave,
            files::discard_recovery,
            files::recoverable,
            files::recover,
            native_menu::set_native_menu
        ])
        .run(tauri::generate_context!())
        .expect("error while running Stylus");
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_save_never_replaces_unasked_and_replaces_whole() {
        let dir = std::env::temp_dir().join(format!("stylus-write-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("art.ans");
        super::write_file(&path, b"one", false).unwrap();
        assert!(super::write_file(&path, b"two", false).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"one");
        super::write_file(&path, b"three", true).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"three");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1, "no temporary file left");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn today_is_a_sauce_date() {
        let t = super::today();
        assert_eq!(t.len(), 8);
        assert!(t.starts_with("20"));
        assert!(t.chars().all(|c| c.is_ascii_digit()));
    }
}
