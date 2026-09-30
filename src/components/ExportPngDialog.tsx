/**
 * File → Export PNG… (docs/roadmap.md, Phase 1c): the art at its native
 * pixel size, 8- or 9-px cells, and optional aspect correction, starting
 * from how it's shown now. The export itself runs in App under a progress
 * bar that counts rows.
 */
import { useEffect, useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { Dialog } from "./Dialog";
import { pngSize, type OpenedArt, type PngOptions } from "../lib/backend";

interface ExportPngDialogProps {
  open: boolean;
  art: OpenedArt;
  onClose: () => void;
  /** A file was picked (the system dialog asked before replacing one). */
  onExport: (path: string, options: PngOptions) => void;
  onError: (message: string) => void;
}

function stemOf(name: string): string {
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(0, dot) : name;
}

export function ExportPngDialog({ open, art, onClose, onExport, onError }: ExportPngDialogProps) {
  const settings = art.info.settings;
  const [options, setOptions] = useState<PngOptions>({ letterSpacing: settings.letterSpacing, aspectRatio: settings.aspectRatio });
  const [size, setSize] = useState<{ width: number; height: number } | null>(null);

  // Only fonts 8 px wide get a 9th column.
  const fontWidth = settings.letterSpacing && art.info.cellWidth === 9 ? 8 : art.info.cellWidth;
  const canSpace = fontWidth === 8;

  // Start from the view each time the dialog opens.
  useEffect(() => {
    if (open) setOptions({ letterSpacing: settings.letterSpacing, aspectRatio: settings.aspectRatio });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, art.id]);

  useEffect(() => {
    if (!open) return;
    let current = true;
    pngSize(art.id, options).then(
      (s) => current && setSize(s),
      (e) => current && onError(String(e)),
    );
    return () => {
      current = false;
    };
  }, [open, art.id, art.info.rows, art.info.columns, options, onError]);

  async function chooseWhere() {
    try {
      const picked = await save({ defaultPath: `${stemOf(art.name)}.png`, filters: [{ name: "PNG image", extensions: ["png"] }] });
      if (!picked) return;
      onExport(picked.toLowerCase().endsWith(".png") ? picked : `${picked}.png`, options);
    } catch (e) {
      onError(`Couldn't export: ${e}`);
    }
  }

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title="Export PNG"
      actions={
        <>
          <button type="button" onClick={onClose}>
            Cancel
          </button>
          <button type="button" className="primary" data-testid="export-png-confirm" disabled={!size} onClick={() => void chooseWhere()}>
            Choose where…
          </button>
        </>
      }
    >
      <p>The art at its own pixel size, in its own colors{settings.iceColors ? ", with iCE colors" : ""}.</p>
      <fieldset className="option-group" disabled={!canSpace}>
        <legend>Letter spacing</legend>
        <label className="toggle">
          <input type="radio" name="png-spacing" checked={!options.letterSpacing || !canSpace} onChange={() => setOptions({ ...options, letterSpacing: false })} />
          <span>{canSpace ? "8 px" : `${fontWidth} px (the font's own width)`}</span>
        </label>
        {canSpace && (
          <label className="toggle">
            <input type="radio" name="png-spacing" data-testid="export-png-9px" checked={options.letterSpacing} onChange={() => setOptions({ ...options, letterSpacing: true })} />
            <span>9 px, as VGA text mode showed it</span>
          </label>
        )}
      </fieldset>
      <label className="toggle">
        <input type="checkbox" data-testid="export-png-aspect" checked={options.aspectRatio} onChange={(e) => setOptions({ ...options, aspectRatio: e.currentTarget.checked })} />
        <span>Aspect correction (taller, as on the original screen)</span>
      </label>
      <p className="muted" data-testid="export-png-size">
        {size ? `${size.width} × ${size.height} pixels` : "Working out the size…"}
      </p>
    </Dialog>
  );
}
