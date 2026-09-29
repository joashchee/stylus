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
 */
import { useEffect, useRef, useState } from "react";
import { renderBand, type DocumentInfo } from "../lib/backend";
import type { ActivityUpdate } from "../lib/activity";

/** Band height in pixels; each band is one canvas. */
const BAND_PX = 2048;
const BLINK_MS = 500;

export type Zoom = "fit" | 1 | 2 | 3;

type RunActivity = <T>(label: string, task: (update: ActivityUpdate) => Promise<T>, key?: string) => Promise<T>;

interface ArtViewerProps {
  id: number;
  name: string;
  info: DocumentInfo;
  zoom: Zoom;
  runActivity: RunActivity;
  onError: (message: string) => void;
}

function prefersReducedMotion() {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

export function ArtViewer({ id, name, info, zoom, runActivity, onError }: ArtViewerProps) {
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
    // `info` is a new object whenever the art or its settings change.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id, info]);

  useEffect(() => () => void generation.current++, []);

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

  return (
    <div className="art-area" ref={areaRef} data-testid="art-area">
      <div className="art-stack" style={{ width: displayWidth }} aria-label={`${name}, ${info.columns} by ${info.rows} characters`} role="img">
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
      </div>
    </div>
  );
}
