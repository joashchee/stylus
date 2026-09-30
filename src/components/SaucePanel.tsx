/**
 * The file's SAUCE record, every field, always shown (Diskette's
 * docs/stylus-notes.md, "Opens"). Files without one say so, and which
 * defaults Stylus opened them with. A save writes the fields from the
 * Save dialog (`SaveDialog`).
 */
import type { DocumentInfo } from "../lib/backend";

interface SaucePanelProps {
  name: string;
  info: DocumentInfo;
  /** Inside another panel (the editor's context panel): no frame of its own. */
  embedded?: boolean;
}

function Row({ label, value }: { label: string; value: string | number | null | undefined }) {
  if (value === null || value === undefined || value === "") return null;
  return (
    <>
      <dt>{label}</dt>
      <dd>{value}</dd>
    </>
  );
}

export function SaucePanel({ name, info, embedded }: SaucePanelProps) {
  const sauce = info.sauce;
  return (
    <aside className={embedded ? "sauce-panel embedded" : "panel sauce-panel"} data-testid="sauce-panel">
      {!embedded && <h2>SAUCE</h2>}
      {sauce ? (
        <dl className="info-list">
          <Row label="Title" value={sauce.title || "(none)"} />
          <Row label="Author" value={sauce.author || "(none)"} />
          <Row label="Group" value={sauce.group || "(none)"} />
          <Row label="Date" value={sauce.date} />
          <Row label="Type" value={sauce.kind} />
          <Row label="Size" value={sauce.columns !== null ? `${sauce.columns} × ${sauce.lines} characters` : null} />
          <Row label="iCE colors" value={sauce.iceColors === null ? null : sauce.iceColors ? "On" : "Off"} />
          <Row label="Letter spacing" value={sauce.letterSpacing} />
          <Row label="Aspect ratio" value={sauce.aspectRatio} />
          <Row label="Font" value={sauce.font} />
          <Row label="File size" value={`${sauce.fileSize.toLocaleString()} bytes`} />
        </dl>
      ) : (
        <p className="desc" data-testid="no-sauce">
          {name} has no SAUCE record. Stylus opened it with the defaults for {info.format}.
        </p>
      )}
      {sauce && sauce.comments.length > 0 && (
        <>
          <h3 className="info-subhead">Comments</h3>
          <pre className="sauce-comments">{sauce.comments.join("\n")}</pre>
        </>
      )}
      <h3 className="info-subhead">As shown</h3>
      <dl className="info-list">
        <Row label="Format" value={info.format} />
        <Row label="Size" value={`${info.columns} × ${info.rows} characters`} />
        <Row label="Font" value={info.font} />
        <Row label="Cell" value={`${info.cellWidth} × ${info.cellHeight} px`} />
        <Row label="Image" value={`${info.pixelWidth} × ${info.pixelHeight} px`} />
      </dl>
    </aside>
  );
}
