/**
 * Draws a whole piece of art, scaled to its container's width: for the
 * Image to ANSI results, where many small pieces show at once. Unlike
 * ArtViewer it has no zoom, blink or activity of its own (the panel's
 * progress bar covers the conversions); it renders every band into its own
 * canvas, like ArtViewer, so a long result never exceeds a canvas's size
 * limit.
 *
 * The art's own colors, always: the theme never touches the canvas.
 */
import { useEffect, useRef } from "react";
import { renderBand, type DocumentInfo } from "../lib/backend";

const BAND_PX = 2048;

interface ArtThumbProps {
  id: number;
  label: string;
  info: DocumentInfo;
  onError: (message: string) => void;
}

export function ArtThumb({ id, label, info, onError }: ArtThumbProps) {
  const rowsPerBand = Math.max(1, Math.floor(BAND_PX / info.cellHeight));
  const bands = Array.from({ length: Math.ceil(info.rows / rowsPerBand) }, (_, i) => {
    const firstRow = i * rowsPerBand;
    return { firstRow, rows: Math.min(rowsPerBand, info.rows - firstRow) };
  });
  const canvases = useRef<(HTMLCanvasElement | null)[]>([]);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      for (let i = 0; i < bands.length; i++) {
        const band = await renderBand(id, bands[i].firstRow, bands[i].rows, true);
        if (cancelled) return;
        const canvas = canvases.current[i];
        if (canvas && band.width > 0) canvas.getContext("2d")?.putImageData(new ImageData(band.rgba, band.width, band.height), 0, 0);
      }
    })().catch((e) => onError(`Couldn't draw ${label}: ${e}`));
    return () => {
      cancelled = true;
    };
    // `info` is a new object whenever the art changes.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id, info]);

  return (
    <div className="art-thumb" role="img" aria-label={`${label}, ${info.columns} by ${info.rows} characters`}>
      {bands.map((band, i) => (
        <canvas
          key={`${id}-${i}`}
          ref={(el) => {
            canvases.current[i] = el;
          }}
          width={info.pixelWidth}
          height={band.rows * info.cellHeight}
          style={{ aspectRatio: `${info.pixelWidth} / ${band.rows * info.cellHeight * info.aspectStretch}` }}
        />
      ))}
    </div>
  );
}
