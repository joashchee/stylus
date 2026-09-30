/**
 * Save and Save As (docs/roadmap.md, Phase 1c). One dialog for both:
 *
 * - The format, and what saving in it would lose, listed before anything
 *   is written ("A save warns before it loses anything"). A loss that makes
 *   the save impossible (BIN wider than 255 columns) disables it.
 * - The SAUCE text fields, starting from the file's own record. The rest of
 *   the record (size, iCE, spacing, font) comes from the document.
 * - Save As picks a new file in the system dialog, which asks itself
 *   before replacing one. Save to the document's own file says it will
 *   replace it (CLAUDE.md rule 4: only when the user asks).
 */
import { useEffect, useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { Dialog } from "./Dialog";
import { saveArt, saveLosses, type DocumentInfo, type OpenedArt, type SauceFields, type SaveFormat, type SaveLoss } from "../lib/backend";

interface SaveDialogProps {
  open: boolean;
  art: OpenedArt;
  /** The document's own file, for Save; null for Save As. */
  path: string | null;
  formats: SaveFormat[];
  onClose: () => void;
  onSaved: (info: DocumentInfo, path: string) => void;
  onError: (message: string) => void;
}

const LIMITS = { title: 35, author: 20, group: 20, comment: 64 };

function extensionOf(path: string): string {
  return path.split(".").pop()?.toLowerCase() ?? "";
}

function stemOf(name: string): string {
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(0, dot) : name;
}

/** The SAUCE fields a save starts from: the file's record, or empty. */
export function sauceFieldsOf(info: DocumentInfo): SauceFields {
  const s = info.sauce;
  return {
    title: s?.title ?? "",
    author: s?.author ?? "",
    group: s?.group ?? "",
    date: s?.date.replace(/\//g, "") ?? "",
    comments: s?.comments ?? [],
  };
}

export function SaveDialog({ open, art, path, formats, onClose, onSaved, onError }: SaveDialogProps) {
  const ownFormat = path ? extensionOf(path) : extensionOf(art.name);
  const [format, setFormat] = useState(formats.some((f) => f.extension === ownFormat) ? ownFormat : "ans");
  const [fields, setFields] = useState<SauceFields>(() => sauceFieldsOf(art.info));
  const [losses, setLosses] = useState<SaveLoss[] | null>(null);
  const [busy, setBusy] = useState(false);

  // Start from the document each time the dialog opens.
  useEffect(() => {
    if (!open) return;
    setFields(sauceFieldsOf(art.info));
    const own = path ? extensionOf(path) : extensionOf(art.name);
    setFormat(formats.some((f) => f.extension === own) ? own : "ans");
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, art.id]);

  useEffect(() => {
    if (!open) return;
    setLosses(null);
    let current = true;
    saveLosses(art.id, format).then(
      (l) => current && setLosses(l),
      (e) => current && onError(String(e)),
    );
    return () => {
      current = false;
    };
  }, [open, art.id, format, onError]);

  const blocked = losses?.some((l) => l.blocking) ?? true;
  const formatName = formats.find((f) => f.extension === format)?.name ?? format.toUpperCase();

  async function doSave() {
    setBusy(true);
    try {
      let target = path;
      if (!target) {
        const picked = await save({
          defaultPath: `${stemOf(art.name)}.${format}`,
          filters: [{ name: formatName, extensions: [format] }],
        });
        if (!picked) return;
        target = extensionOf(picked) === format ? picked : `${picked}.${format}`;
      }
      // The system dialog asked before replacing; Save replaces the
      // document's own file, which this dialog said it would.
      const comments = [...fields.comments];
      while (comments.length > 0 && comments[comments.length - 1].trim() === "") comments.pop();
      const info = await saveArt(art.id, target, { ...fields, comments }, true);
      onSaved(info, target);
    } catch (e) {
      onError(`Couldn't save: ${e}`);
    } finally {
      setBusy(false);
    }
  }

  const name = path ? path.split(/[\\/]/).pop() : null;

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title={path ? `Save ${name}` : "Save As"}
      className="dialog-wide save-dialog"
      actions={
        <>
          <button type="button" onClick={onClose}>
            Cancel
          </button>
          <button type="button" className="primary" data-testid="save-confirm" disabled={blocked || busy} onClick={() => void doSave()}>
            {path ? `Replace ${name}` : "Choose where…"}
          </button>
        </>
      }
    >
      {path ? (
        <p>This replaces {name} with the art as it is now. To keep the original, use Save As instead.</p>
      ) : (
        <label className="form-row">
          <span>Format</span>
          <select data-testid="save-format" value={format} onChange={(e) => setFormat(e.currentTarget.value)}>
            {formats.map((f) => (
              <option key={f.extension} value={f.extension}>
                {f.name} (.{f.extension.toUpperCase()})
              </option>
            ))}
          </select>
        </label>
      )}

      <div className="save-losses" data-testid="save-losses" role="status">
        {losses === null ? (
          <p className="muted">Checking what {formatName} keeps…</p>
        ) : losses.length === 0 ? (
          <p className="muted">{formatName} keeps everything in this art.</p>
        ) : (
          <>
            <p>{blocked ? `${formatName} can't hold this art:` : `Saving as ${formatName} loses some of this art:`}</p>
            <ul>
              {losses.map((l) => (
                <li key={l.message} className={l.blocking ? "loss-blocking" : undefined}>
                  {l.message}
                </li>
              ))}
            </ul>
          </>
        )}
      </div>

      <fieldset className="sauce-fields">
        <legend>SAUCE</legend>
        <label className="form-row">
          <span>Title</span>
          <input data-testid="sauce-title" maxLength={LIMITS.title} value={fields.title} onChange={(e) => setFields({ ...fields, title: e.currentTarget.value })} />
        </label>
        <label className="form-row">
          <span>Author</span>
          <input data-testid="sauce-author" maxLength={LIMITS.author} value={fields.author} onChange={(e) => setFields({ ...fields, author: e.currentTarget.value })} />
        </label>
        <label className="form-row">
          <span>Group</span>
          <input maxLength={LIMITS.group} value={fields.group} onChange={(e) => setFields({ ...fields, group: e.currentTarget.value })} />
        </label>
        <label className="form-row form-row-top">
          <span>Comments</span>
          <textarea
            rows={3}
            value={fields.comments.join("\n")}
            placeholder={`Up to ${LIMITS.comment} characters a line`}
            onChange={(e) => setFields({ ...fields, comments: e.currentTarget.value.split("\n").map((line) => line.slice(0, LIMITS.comment)) })}
          />
        </label>
        <p className="muted">The size, iCE colors, letter spacing and font are saved from the art itself.</p>
      </fieldset>
    </Dialog>
  );
}
