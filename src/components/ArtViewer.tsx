/**
 * Shows opened art: stylus-core renders it band by band (lib/backend.ts
 * renderBand) and each band is drawn into its own <canvas>, stacked. One
 * canvas per band keeps each under the webview's canvas size limit, which
 * long ANSIs (thousands of rows) would pass.
 *
 * The art's own colors, always: the theme never touches the canvas.
 *
 * - Rendering is a determinate activity counting rows. A newer render
 *   (settings changed, another file opened) stops the older one.
 * - Aspect correction and zoom are display scaling (CSS), not new pixels.
 * - Blink: with iCE off and blinking cells present, a second frame with
 *   those cells hidden is rendered, and the two alternate once a second
 *   (well under the 3-flashes-a-second limit). With reduced motion it
 *   stays on the "on" frame.
 * - Editing (ArtEditor): `redraw` re-renders just the cells an edit
 *   changed, and the pointer callbacks report the cell under the pointer,
 *   from the art's displayed size, so zoom and aspect need no maths here.
 */
import { forwardRef, useEffect, useImperativeHandle, useRef, useState, type PointerEvent as ReactPointerEvent, type ReactNode } from "react";
import { renderBand, renderCells, type CellRect, type DocumentInfo } from "../lib/backend";
import type { ActivityUpdate } from "../lib/activity";

/** Band height in pixels; each band is one canvas. */
const BAND_PX = 2048;
const BLINK_MS = 500;

export type Zoom = "fit" | 1 | 2 | 3 | 4;

type RunActivity = <T>(label: string, task: (update: ActivityUpdate) => Promise<T>, key?: string) => Promise<T>;

export interface Cell {
  x: number;
  y: number;
  /** From the pointer: the row in half cells (0 to 2 × rows − 1), for the half-block brush. */
  half?: number;
}

/** Pointer events on the art, in cells. Only when editing. */
export interface ArtPointerHandlers {
  onCellDown: (cell: Cell, event: ReactPointerEvent<HTMLDivElement>) => void;
  onCellMove: (cell: Cell, event: ReactPointerEvent<HTMLDivElement>) => void;
  onCellUp: (event: ReactPointerEvent<HTMLDivElement>) => void;
  onLeave?: () => void;
}

export interface ArtViewerHandle {
  /** Re-renders a rectangle of cells after an edit. */
  redraw: (rect: CellRect) => Promise<void>;
  /** Scrolls so a cell is visible (the text cursor). */
  reveal: (cell: Cell) => void;
}

interface ArtViewerProps {
  id: number;
  name: string;
  info: DocumentInfo;
  zoom: Zoom;
  runActivity: RunActivity;
  onError: (message: string) => void;
  pointer?: ArtPointerHandlers;
  /** Drawn over the art, positioned in cells (the cursor, a selection). */
  overlay?: ReactNode;
}

function prefersReducedMotion() {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

export const ArtViewer = forwardRef<ArtViewerHandle, ArtViewerProps>(function ArtViewer({ id, name, info, zoom, runActivity, onError, pointer, overlay }, ref) {
  const rowsPerBand = Math.max(1, Math.floor(BAND_PX / info.cellHeight));
  const bandCount = Math.ceil(info.rows / rowsPerBand);
  const bands = Array.from({ length: bandCount }, (_, i) => {
    const firstRow = i * rowsPerBand;
    const rows = Math.min(rowsPerBand, info.rows - firstRow);
    return { firstRow, rows, height: rows * info.cellHeight };
  });
  const animateBlink = info.hasBlink && !info.settings.iceColors;

  const onCanvases = useRef<(HTMLCanvasElement | null)[]>([]);
  const offCanvases = useRef<(HTMLCanvasElement | null)[]>([]);
  const generation = useRef(0);
  const areaRef = useRef<HTMLDivElement>(null);
  const stackRef = useRef<HTMLDivElement>(null);
  const [areaWidth, setAreaWidth] = useState(0);
  const [blinkVisible, setBlinkVisible] = useState(true);

  // Render every band, "on" frames first, then the blink "off" frames.
  useEffect(() => {
    const gen = ++generation.current;
    const frames = animateBlink ? 2 : 1;
    const totalRows = info.rows * frames;
    void runActivity(`Rendering ${name}…`, async (update) => {
      let done = 0;
      for (let frame = 0; frame < frames; frame++) {
        const blinkOn = frame === 0;
        const canvases = blinkOn ? onCanvases.current : offCanvases.current;
        for (let i = 0; i < bands.length; i++) {
          const band = await renderBand(id, bands[i].firstRow, bands[i].rows, blinkOn);
          if (gen !== generation.current) return;
          const canvas = canvases[i];
          if (canvas && band.width > 0) canvas.getContext("2d")?.putImageData(new ImageData(band.rgba, band.width, band.height), 0, 0);
          done += bands[i].rows;
          update({ value: done / totalRows });
        }
      }
    }).catch((e) => onError(`Couldn't render ${name}: ${e}`));
    // Only what changes the pixels: an edit redraws its own cells (`redraw`),
    // and a save or the aspect setting needs no new render.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id, info.columns, info.rows, info.settings.letterSpacing, info.settings.iceColors, info.font, info.hasBlink]);

  useEffect(() => () => void generation.current++, []);

  useImperativeHandle(
    ref,
    () => ({
      async redraw(rect: CellRect) {
        const frames = animateBlink ? [true, false] : [true];
        for (let i = 0; i < bands.length; i++) {
          const band = bands[i];
          const top = Math.max(rect.y, band.firstRow);
          const bottom = Math.min(rect.y + rect.height, band.firstRow + band.rows);
          if (top >= bottom) continue;
          for (const blinkOn of frames) {
            const image = await renderCells(id, { x: rect.x, y: top, width: rect.width, height: bottom - top }, blinkOn);
            const canvas = (blinkOn ? onCanvases.current : offCanvases.current)[i];
            if (canvas && image.width > 0) {
              canvas.getContext("2d")?.putImageData(new ImageData(image.rgba, image.width, image.height), rect.x * info.cellWidth, (top - band.firstRow) * info.cellHeight);
            }
          }
        }
      },
      reveal(cell: Cell) {
        const area = areaRef.current;
        const stack = stackRef.current;
        if (!area || !stack) return;
        const cellW = stack.offsetWidth / info.columns;
        const cellH = stack.offsetHeight / info.rows;
        const left = stack.offsetLeft + cell.x * cellW;
        const top = cell.y * cellH;
        if (left < area.scrollLeft) area.scrollLeft = left;
        else if (left + cellW > area.scrollLeft + area.clientWidth) area.scrollLeft = left + cellW - area.clientWidth;
        if (top < area.scrollTop) area.scrollTop = top;
        else if (top + cellH > area.scrollTop + area.clientHeight) area.scrollTop = top + cellH - area.clientHeight;
      },
    }),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [id, info, animateBlink],
  );

  useEffect(() => {
    if (!animateBlink || prefersReducedMotion()) {
      setBlinkVisible(true);
      return;
    }
    const timer = window.setInterval(() => setBlinkVisible((v) => !v), BLINK_MS);
    return () => window.clearInterval(timer);
  }, [animateBlink]);

  useEffect(() => {
    const el = areaRef.current;
    if (!el) return;
    const observer = new ResizeObserver(([entry]) => setAreaWidth(entry.contentRect.width));
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const scale = zoom === "fit" ? (areaWidth > 0 ? areaWidth / info.pixelWidth : 1) : zoom;
  const displayWidth = info.pixelWidth * scale;
  // Pixelated when enlarging keeps cells crisp; shrinking reads better smoothed.
  const rendering = scale >= 1 ? "pixelated" : "auto";

  function cellAt(event: ReactPointerEvent<HTMLDivElement>): Cell {
    const rect = event.currentTarget.getBoundingClientRect();
    const x = Math.floor(((event.clientX - rect.left) / rect.width) * info.columns);
    const fy = (event.clientY - rect.top) / rect.height;
    const y = Math.floor(fy * info.rows);
    const half = Math.min(Math.max(Math.floor(fy * info.rows * 2), 0), info.rows * 2 - 1);
    return { x: Math.min(Math.max(x, 0), info.columns - 1), y: Math.min(Math.max(y, 0), info.rows - 1), half };
  }

  return (
    <div className="art-area" ref={areaRef} data-testid="art-area">
      <div
        ref={stackRef}
        className={`art-stack${pointer ? " editing" : ""}`}
        style={{ width: displayWidth }}
        aria-label={`${name}, ${info.columns} by ${info.rows} characters`}
        role="img"
        onPointerDown={
          pointer &&
          ((e) => {
            e.currentTarget.setPointerCapture(e.pointerId);
            pointer.onCellDown(cellAt(e), e);
          })
        }
        onPointerMove={pointer && ((e) => pointer.onCellMove(cellAt(e), e))}
        onPointerUp={pointer && ((e) => pointer.onCellUp(e))}
        onPointerLeave={pointer?.onLeave}
        onContextMenu={pointer && ((e) => e.preventDefault())}
      >
        {bands.map((band, i) => (
          <div key={`${id}-${i}`} className="art-band" style={{ height: band.height * scale * info.aspectStretch }}>
            <canvas
              ref={(el) => {
                onCanvases.current[i] = el;
              }}
              width={info.pixelWidth}
              height={band.height}
              style={{ imageRendering: rendering }}
            />
            {animateBlink && (
              <canvas
                ref={(el) => {
                  offCanvases.current[i] = el;
                }}
                className={blinkVisible ? "art-blink-off" : "art-blink-off shown"}
                width={info.pixelWidth}
                height={band.height}
                style={{ imageRendering: rendering }}
              />
            )}
          </div>
        ))}
        {overlay}
      </div>
    </div>
  );
});

/** A box over one or more cells, positioned in percent of the art. */
export function CellBox({ info, rect, className }: { info: DocumentInfo; rect: CellRect; className: string }) {
  return (
    <div
      className={className}
      aria-hidden="true"
      style={{
        left: `${(rect.x / info.columns) * 100}%`,
        top: `${(rect.y / info.rows) * 100}%`,
        width: `${(rect.width / info.columns) * 100}%`,
        height: `${(rect.height / info.rows) * 100}%`,
      }}
    />
  );
}
