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
