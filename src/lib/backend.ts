/**
 * The one frontend adapter over stylus-core (Diskette's
 * docs/stylus-notes.md, "Architecture"). Every call into the core goes
 * through here, so desktop and web share the rest of the frontend:
 *
 * - Desktop: Tauri commands (src-tauri/src/lib.rs) wrap the core, for
 *   file I/O and OS integration.
 * - Web (later, build step 7): the same core compiled to WebAssembly,
 *   picked at build time.
 *
 * The editor's hot path will run the WASM core in the webview on desktop
 * too, since a keystroke can't wait on an IPC round trip per cell. The
 * core builds for wasm32 (icy_tools 2e4e2b1); only the Tauri side exists
 * so far.
 */
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface CoreInfo {
  /** stylus-core's crate version. */
  version: string;
  /** The icy_tools git revision stylus-core is pinned to. */
  icyToolsRev: string;
}

export function coreInfo(): Promise<CoreInfo> {
  return invoke<CoreInfo>("core_info");
}

/** How the art is shown; mirrors stylus-core's `RenderSettings`. */
export interface RenderSettings {
  /** 9-px character cells instead of 8-px. */
  letterSpacing: boolean;
  /** iCE colors: bright backgrounds instead of blink. */
  iceColors: boolean;
  /** Stretch to the original display's aspect (`DocumentInfo.aspectStretch`). */
  aspectRatio: boolean;
}

/** Every SAUCE field, decoded from CP437; mirrors stylus-core's `SauceInfo`. */
export interface SauceInfo {
  title: string;
  author: string;
  group: string;
  date: string;
  comments: string[];
  kind: string;
  fileSize: number;
  columns: number | null;
  lines: number | null;
  iceColors: boolean | null;
  letterSpacing: string | null;
  aspectRatio: string | null;
  font: string | null;
}

export interface DocumentInfo {
  columns: number;
  rows: number;
  format: string;
  font: string;
  cellWidth: number;
  cellHeight: number;
  pixelWidth: number;
  pixelHeight: number;
  aspectStretch: number;
  hasBlink: boolean;
  settings: RenderSettings;
  sauce: SauceInfo | null;
  canUndo: boolean;
  canRedo: boolean;
  /** Changed since it was opened or last saved. */
  edited: boolean;
}

export interface OpenedArt {
  id: number;
  name: string;
  info: DocumentInfo;
}

export interface Band {
  width: number;
  height: number;
  rgba: Uint8ClampedArray<ArrayBuffer>;
}

export function openExtensions(): Promise<string[]> {
  return invoke<string[]>("open_extensions");
}

/** Reads and opens a file. Never changes it. */
export function openArt(path: string): Promise<OpenedArt> {
  return invoke<OpenedArt>("open_art", { path });
}

export function closeArt(id: number): Promise<void> {
  return invoke("close_art", { id });
}

export function setRenderSettings(id: number, settings: RenderSettings): Promise<DocumentInfo> {
  return invoke<DocumentInfo>("set_render_settings", { id, settings });
}

/** Rows `firstRow .. firstRow + rowCount` as RGBA. `blinkOn` false hides blinking cells. */
export async function renderBand(id: number, firstRow: number, rowCount: number, blinkOn: boolean): Promise<Band> {
  const buffer = await invoke<ArrayBuffer>("render_band", { id, firstRow, rowCount, blinkOn });
  const header = new DataView(buffer, 0, 8);
  return {
    width: header.getUint32(0, true),
    height: header.getUint32(4, true),
    rgba: new Uint8ClampedArray(buffer, 8),
  };
}

/** Where an image-to-ANSI converter came from and how it's run; mirrors stylus-core's `ConverterInfo`. */
export interface ConverterInfo {
  /** Stable id, used in saved file names. */
  id: string;
  /** The project's own name. */
  name: string;
  /** The repository it's from, e.g. "github.com/koan-shdw/koan-ansi". */
  origin: string;
  copyright: string;
  /** SPDX license id. */
  license: string;
  language: string;
  /** The commit ported. */
  revision: string;
  /** The original's settings used, in its own terms. */
  settings: string;
  /** What the port changes from the original, and why. */
  adaptations: string;
}

export interface LoadedImage {
  id: number;
  name: string;
  width: number;
  height: number;
}

export function imageExtensions(): Promise<string[]> {
  return invoke<string[]>("image_extensions");
}

export function listConverters(): Promise<ConverterInfo[]> {
  return invoke<ConverterInfo[]>("list_converters");
}

/** A converter's full license text. */
export function converterLicense(id: string): Promise<string> {
  return invoke<string>("converter_license", { id });
}

/** Reads and decodes an image for Image to ANSI. Never changes it. */
export function loadImage(path: string): Promise<LoadedImage> {
  return invoke<LoadedImage>("load_image", { path });
}

export function closeImage(id: number): Promise<void> {
  return invoke("close_image", { id });
}

/** Converts a loaded image with one converter; the .ANS it makes opens as art. */
export function convertImage(imageId: number, converterId: string): Promise<OpenedArt> {
  return invoke<OpenedArt>("convert_image", { imageId, converterId });
}

/**
 * Writes converted art to `path`. With `replace` false an existing file is
 * left alone and the call fails, so saving never overwrites unasked.
 */
export function saveMadeArt(id: number, path: string, replace: boolean): Promise<void> {
  return invoke("save_made_art", { id, path, replace });
}

/** Library code the converter ports carry, with its license; mirrors stylus-core's `LibraryNotice`. */
export interface LibraryNotice {
  name: string;
  origin: string;
  license: string;
  usedFor: string;
  licenseText: string;
}

export function libraryNotices(): Promise<LibraryNotice[]> {
  return invoke<LibraryNotice[]>("library_notices");
}

/** A rectangle of cells; mirrors stylus-core's `CellRect`. */
export interface CellRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** One cell to change; a field left out keeps what the cell has. Mirrors `CellEdit`. */
export interface CellEdit {
  x: number;
  y: number;
  /** Character code in the document's font (CP437 for the IBM fonts). */
  code?: number;
  fg?: number;
  bg?: number;
}

/** What a cell holds; mirrors stylus-core's `CellInfo`. */
export interface CellInfo {
  code: number;
  fg: number;
  bg: number;
  blink: boolean;
  /** A 24-bit color, so fg/bg are the nearest of the 16. */
  truecolor: boolean;
}

/** What an edit, undo or redo changed. `info` is set when the size changed. */
export interface EditResult {
  dirty: CellRect | null;
  canUndo: boolean;
  canRedo: boolean;
  edited: boolean;
  info: DocumentInfo | null;
}

export interface SaveFormat {
  extension: string;
  name: string;
}

/** Something a save would lose; `blocking` means it can't save at all. */
export interface SaveLoss {
  blocking: boolean;
  message: string;
}

/** The SAUCE text a save writes; mirrors stylus-core's `SauceFields`. */
export interface SauceFields {
  title: string;
  author: string;
  group: string;
  /** CCYYMMDD; empty means today. */
  date: string;
  comments: string[];
}

export function newArt(columns: number, rows: number, iceColors: boolean): Promise<OpenedArt> {
  return invoke<OpenedArt>("new_art", { columns, rows, iceColors });
}

export function documentInfo(id: number): Promise<DocumentInfo> {
  return invoke<DocumentInfo>("document_info", { id });
}

/** Applies edits as part of stroke `stroke`: the same id undoes together. */
export function applyEdits(id: number, stroke: number, edits: CellEdit[]): Promise<EditResult> {
  return invoke<EditResult>("apply_edits", { id, stroke, edits });
}

export function undo(id: number): Promise<EditResult> {
  return invoke<EditResult>("undo", { id });
}

export function redo(id: number): Promise<EditResult> {
  return invoke<EditResult>("redo", { id });
}

export function resizeArt(id: number, columns: number, rows: number): Promise<DocumentInfo> {
  return invoke<DocumentInfo>("resize_art", { id, columns, rows });
}

export function cellAt(id: number, x: number, y: number): Promise<CellInfo | null> {
  return invoke<CellInfo | null>("cell_at", { id, x, y });
}

/** A rectangle of cells as RGBA, for redrawing what an edit changed. */
export async function renderCells(id: number, rect: CellRect, blinkOn: boolean): Promise<Band> {
  const buffer = await invoke<ArrayBuffer>("render_cells", { id, rect, blinkOn });
  const header = new DataView(buffer, 0, 8);
  return {
    width: header.getUint32(0, true),
    height: header.getUint32(4, true),
    rgba: new Uint8ClampedArray(buffer, 8),
  };
}

export function saveFormats(): Promise<SaveFormat[]> {
  return invoke<SaveFormat[]>("save_formats");
}

export function saveLosses(id: number, extension: string): Promise<SaveLoss[]> {
  return invoke<SaveLoss[]>("save_losses", { id, extension });
}

/**
 * Saves to `path` in the format its extension names. With `replace` false an
 * existing file is left alone and the call fails; with it true the file is
 * replaced whole (written beside it, then moved over it).
 */
export function saveArt(id: number, path: string, sauce: SauceFields, replace: boolean): Promise<DocumentInfo> {
  return invoke<DocumentInfo>("save_art", { id, path, sauce, replace });
}

/** How to export a PNG; mirrors stylus-core's `PngOptions`. iCE follows the document. */
export interface PngOptions {
  /** 9-px cells (fonts 8 px wide only). */
  letterSpacing: boolean;
  /** Stretch to the original display's aspect. */
  aspectRatio: boolean;
}

export function pngSize(id: number, options: PngOptions): Promise<{ width: number; height: number }> {
  return invoke("png_size", { id, options });
}

/** Rows written per call: small enough for a smooth bar, large enough that IPC doesn't dominate. */
const PNG_EXPORT_ROWS = 50;

/**
 * Exports the art as a PNG at `path`, which the system dialog has already
 * cleared to replace. `onProgress` gets 0–1 as rows are written. Nothing
 * is left at `path` if it fails.
 */
export async function exportPng(id: number, path: string, options: PngOptions, onProgress: (value: number) => void): Promise<void> {
  const { export: job, rows } = await invoke<{ export: number; rows: number }>("begin_png_export", { id, path, options });
  let done = 0;
  try {
    // The call that writes the last row also finishes the file.
    for (;;) {
      done = await invoke<number>("png_export_rows", { export: job, rows: PNG_EXPORT_ROWS });
      onProgress(rows > 0 ? done / rows : 1);
      if (done >= rows) break;
    }
  } catch (e) {
    // A failed call has already removed the export; this is for anything else.
    await invoke("cancel_png_export", { export: job }).catch(() => undefined);
    throw e;
  }
}

export function textFonts(): Promise<string[]> {
  return invoke<string[]>("text_fonts");
}

/** Shows a text file in another font (Amiga ASCII). Not an edit. */
export function setTextFont(id: number, name: string): Promise<DocumentInfo> {
  return invoke<DocumentInfo>("set_text_font", { id, name });
}

/** A cell position; mirrors stylus-core's `Point`. */
export interface Point {
  x: number;
  y: number;
}

/** What a tool paints; a field left out keeps what the cell has. Mirrors `Pen`. */
export interface Pen {
  code?: number;
  fg?: number;
  bg?: number;
}

/** A dragged shape; mirrors stylus-core's `Shape`. */
export type Shape =
  | { kind: "line"; from: Point; to: Point }
  | { kind: "rectangle"; from: Point; to: Point; filled: boolean }
  | { kind: "box"; from: Point; to: Point; double: boolean };

/** Something done to the selection; mirrors stylus-core's `SelectionOp`. */
export type SelectionOp =
  | { kind: "clear" }
  | { kind: "fill"; pen: Pen }
  | { kind: "flipHorizontal" }
  | { kind: "flipVertical" }
  | { kind: "move"; to: Point };

/**
 * Draws a shape as stroke `stroke`, replacing what the same stroke drew
 * before: call it at each move of a drag, so the preview is the art itself
 * and the drag is one undo step.
 */
export function drawShape(id: number, stroke: number, shape: Shape, pen: Pen): Promise<EditResult> {
  return invoke<EditResult>("draw_shape", { id, stroke, shape, pen });
}

/** Fills the cell at `at` and every joined cell exactly like it. */
export function floodFill(id: number, stroke: number, at: Point, pen: Pen): Promise<EditResult> {
  return invoke<EditResult>("flood_fill", { id, stroke, at, pen });
}

/** The half-block brush along a line; `y` counts half rows. */
export function halfBlock(id: number, stroke: number, from: Point, to: Point, color: number): Promise<EditResult> {
  return invoke<EditResult>("half_block", { id, stroke, from, to, color });
}

export function selectionOp(id: number, stroke: number, rect: CellRect, op: SelectionOp): Promise<EditResult> {
  return invoke<EditResult>("selection_op", { id, stroke, rect, op });
}

/** Copies cells to Stylus's clipboard (shared by every open document). Returns its size. */
export function copyCells(id: number, rect: CellRect): Promise<{ width: number; height: number } | null> {
  return invoke<{ width: number; height: number } | null>("copy_cells", { id, rect });
}

/** Pastes Stylus's clipboard at `at`; returns what changed and where the paste landed. */
export function pasteCells(id: number, stroke: number, at: Point, transparent: boolean): Promise<[EditResult, CellRect | null]> {
  return invoke<[EditResult, CellRect | null]>("paste_cells", { id, stroke, at, transparent });
}

/** An item of the macOS menu bar; mirrors src-tauri's `native_menu::NativeItem`. */
export interface NativeItem {
  id: string;
  label: string;
  /** The registry's form (`mod+shift+s`); only ⌘ shortcuts reach the menu. */
  shortcut?: string;
  enabled: boolean;
  /** Present for items with a tick. */
  checked?: boolean;
  /** A submenu's items (`null` is a separator). */
  children?: (NativeItem | null)[];
}

export interface NativeMenuSpec {
  /** `null` is a separator. */
  menus: { label: string; items: (NativeItem | null)[] }[];
  /** Where the keyboard is: the art, a text field, or a dialog. */
  focus: "art" | "text" | "modal";
}

/** Builds or updates the macOS menu bar (a no-op elsewhere). */
export function setNativeMenu(spec: NativeMenuSpec): Promise<void> {
  return invoke("set_native_menu", { spec });
}

/** Calls back with a command's id when it's chosen in the macOS menu bar. */
export function onNativeMenu(run: (id: string) => void): Promise<() => void> {
  return listen<string>("native-menu", (e) => run(e.payload));
}

/** The recent files, newest first. */
export function recentFiles(): Promise<string[]> {
  return invoke<string[]>("recent_files");
}

/** Puts a file just opened or saved at the top of the recent files. */
export function noteRecent(path: string): Promise<string[]> {
  return invoke<string[]>("note_recent", { path });
}

/** Takes `path` off the recent files (only if it's gone, with `onlyIfMissing`); `null` clears them. */
export function forgetRecent(path: string | null, onlyIfMissing = false): Promise<string[]> {
  return invoke<string[]>("forget_recent", { path, onlyIfMissing });
}

/** An autosave left from when Stylus last stopped; mirrors src-tauri's `files::Recoverable`. */
export interface Recoverable {
  key: string;
  name: string;
  path: string | null;
  /** Seconds since 1970. */
  savedAt: number;
}

/**
 * Autosaves document `id` to the app-data folder under `key` if it has
 * unsaved changes the last autosave didn't; removes its autosave if it has
 * none. Never touches the user's file. Resolves to whether it wrote.
 */
export function autosave(id: number, key: string, name: string, path: string | null): Promise<boolean> {
  return invoke<boolean>("autosave", { id, key, name, path });
}

/** Removes a document's autosave (saved, discarded or closed). */
export function discardRecovery(key: string): Promise<void> {
  return invoke("discard_recovery", { key });
}

export function recoverable(): Promise<Recoverable[]> {
  return invoke<Recoverable[]>("recoverable");
}

/** Opens an autosave as art with unsaved changes. */
export function recover(key: string): Promise<OpenedArt> {
  return invoke<OpenedArt>("recover", { key });
}
