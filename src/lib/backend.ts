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
 * too, since a keystroke can't wait on an IPC round trip per cell. Until
 * icy_engine builds for wasm32 (icy_tools#186), only the Tauri side exists.
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
