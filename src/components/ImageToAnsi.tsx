/**
 * Image to ANSI: drop or choose an image and every image-to-ANSI converter
 * Stylus carries (stylus-core's `convert` module, ports of open-source
 * converters whose licenses allow closed-source commercial use) turns it
 * into an .ANS file, shown side by side, each labelled with where it's
 * from, for comparing them.
 *
 * Conversions run a few at a time with one determinate progress bar
 * counting converters. Each result is .ANS bytes held by the backend until
 * saved: "Save .ANS…" writes one (the save dialog asks before replacing),
 * "Save all…" writes every one into a folder and never replaces a file
 * that's already there (CLAUDE.md rule 4). The image itself is only read.
 */
import { useCallback, useEffect, useRef, useState } from "react";
import { join } from "@tauri-apps/api/path";
import { open, save } from "@tauri-apps/plugin-dialog";
import { ArtThumb } from "./ArtThumb";
import { FolderIcon } from "./icons";
import type { ActivityUpdate } from "../lib/activity";
import { closeArt, closeImage, convertImage, loadImage, saveMadeArt, type ConverterInfo, type LoadedImage, type OpenedArt } from "../lib/backend";

type RunActivity = <T>(label: string, task: (update: ActivityUpdate) => Promise<T>, key?: string) => Promise<T>;

/** Conversions in flight at once. */
const PARALLEL = 3;

interface Result {
  converter: ConverterInfo;
  art?: OpenedArt;
  error?: string;
}

interface ImageToAnsiProps {
  converters: ConverterInfo[];
  extensions: string[];
  /** An image path dropped on the window, to convert. */
  droppedPath: { path: string; at: number } | null;
  runActivity: RunActivity;
  onError: (message: string | null) => void;
  onStatus: (message: string | null) => void;
}

export function ImageToAnsi({ converters, extensions, droppedPath, runActivity, onError, onStatus }: ImageToAnsiProps) {
  const [image, setImage] = useState<LoadedImage | null>(null);
  const [results, setResults] = useState<Result[]>([]);
  const [busy, setBusy] = useState(false);
  const generation = useRef(0);
  const openIds = useRef<{ image: number | null; arts: number[] }>({ image: null, arts: [] });

  const release = () => {
    const { image: imageId, arts } = openIds.current;
    if (imageId !== null) void closeImage(imageId);
    arts.forEach((id) => void closeArt(id));
    openIds.current = { image: null, arts: [] };
  };

  useEffect(() => release, []);

  const convertPath = useCallback(
    async (path: string) => {
      const gen = ++generation.current;
      onError(null);
      onStatus(null);
      release();
      setImage(null);
      setResults(converters.map((converter) => ({ converter })));
      setBusy(true);
      const name = path.split(/[\\/]/).pop() ?? path;
      try {
        await runActivity(`Converting ${name}…`, async (update) => {
          const loaded = await loadImage(path);
          if (gen !== generation.current) return void closeImage(loaded.id);
          openIds.current.image = loaded.id;
          setImage(loaded);
          let done = 0;
          const queue = [...converters];
          const worker = async () => {
            for (let c = queue.shift(); c; c = queue.shift()) {
              const converter = c;
              let result: Result;
              try {
                const art = await convertImage(loaded.id, converter.id);
                if (gen !== generation.current) {
                  void closeArt(art.id);
                  return;
                }
                openIds.current.arts.push(art.id);
                result = { converter, art };
              } catch (e) {
                result = { converter, error: String(e) };
              }
              if (gen !== generation.current) return;
              setResults((list) => list.map((r) => (r.converter.id === converter.id ? result : r)));
              done += 1;
              update({ value: done / converters.length, label: `Converting ${name}: ${done} of ${converters.length} converters…` });
            }
          };
          await Promise.all(Array.from({ length: PARALLEL }, worker));
        });
      } catch (e) {
        if (gen === generation.current) {
          onError(String(e));
          setResults([]);
        }
      } finally {
        if (gen === generation.current) setBusy(false);
      }
    },
    [converters, runActivity, onError, onStatus],
  );

  // Each drop once, even when the same file is dropped again.
  const handledDrop = useRef(0);
  useEffect(() => {
    if (droppedPath && droppedPath.at !== handledDrop.current) {
      handledDrop.current = droppedPath.at;
      void convertPath(droppedPath.path);
    }
  }, [droppedPath, convertPath]);

  async function pickAndConvert() {
    const path = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "Images", extensions: extensions.flatMap((e) => [e, e.toUpperCase()]) }],
    });
    if (typeof path === "string") await convertPath(path);
  }

  async function saveOne(art: OpenedArt) {
    const path = await save({ defaultPath: art.name, filters: [{ name: "ANSI art", extensions: ["ans"] }] });
    if (!path) return;
    try {
      // The save dialog has already asked about replacing.
      await saveMadeArt(art.id, path, true);
      onStatus(`Saved ${path.split(/[\\/]/).pop()}.`);
    } catch (e) {
      onError(String(e));
    }
  }

  async function saveAll() {
    const folder = await open({ directory: true, multiple: false, title: "Save every .ANS into a folder" });
    if (typeof folder !== "string") return;
    const arts = results.flatMap((r) => (r.art ? [r.art] : []));
    let saved = 0;
    const skipped: string[] = [];
    await runActivity("Saving the .ANS files…", async (update) => {
      for (const art of arts) {
        try {
          await saveMadeArt(art.id, await join(folder, art.name), false);
          saved += 1;
        } catch {
          skipped.push(art.name);
        }
        update({ value: (saved + skipped.length) / arts.length });
      }
    });
    onStatus(`Saved ${saved} .ANS file${saved === 1 ? "" : "s"}.`);
    if (skipped.length > 0) onError(`Left alone, since files with these names are already there: ${skipped.join(", ")}.`);
  }

  const finished = results.filter((r) => r.art).length;

  return (
    <section className="panel image-to-ansi" data-testid="image-to-ansi">
      <h2>Image to ANSI</h2>
      <p className="desc">
        Drop an image here or choose one. Each of the {converters.length} open-source converters below turns it into an .ANS file, 80 columns wide, labelled with
        where it's from, so you can compare them. The image is only read, never changed.
      </p>
      <div className="viewer-toolbar">
        <button type="button" className="primary icontext-btn" data-testid="choose-image" onClick={() => void pickAndConvert()} disabled={busy}>
          <span className="btn-icon">
            <FolderIcon />
          </span>
          Choose image…
        </button>
        {finished > 0 && (
          <button type="button" className="small" data-testid="save-all" onClick={() => void saveAll()} disabled={busy}>
            Save all…
          </button>
        )}
        {image && (
          <span className="muted mono" data-testid="image-info">
            {image.name}, {image.width} × {image.height}
          </span>
        )}
      </div>

      {results.length > 0 && (
        <ul className="conversion-grid" data-testid="conversion-grid">
          {results.map(({ converter, art, error }) => (
            <li key={converter.id} className="conversion-card" data-testid={`conversion-${converter.id}`}>
              <div className="conversion-art">
                {art ? (
                  <ArtThumb id={art.id} label={converter.name} info={art.info} onError={(m) => onError(m)} />
                ) : error ? (
                  <p className="error">{error}</p>
                ) : (
                  <p className="muted conversion-waiting">Converting…</p>
                )}
              </div>
              <div className="conversion-label">
                <strong>{converter.name}</strong>
                <span className="mono">{converter.origin}</span>
                <span className="muted">
                  {converter.license} · {converter.language}
                  {art && ` · ${art.info.columns} × ${art.info.rows}`}
                </span>
              </div>
              <details className="conversion-details">
                <summary>How it's run here</summary>
                <p>
                  <span className="muted">Settings:</span> {converter.settings}
                </p>
                <p>
                  <span className="muted">Changes from the original:</span> {converter.adaptations}
                </p>
                <p className="mono muted">
                  {converter.copyright}. Ported from commit {converter.revision.slice(0, 12)}.
                </p>
              </details>
              {art && (
                <button type="button" className="small" onClick={() => void saveOne(art)}>
                  Save .ANS…
                </button>
              )}
            </li>
          ))}
        </ul>
      )}
      {results.length === 0 && !busy && <p className="empty">No image yet.</p>}
    </section>
  );
}
