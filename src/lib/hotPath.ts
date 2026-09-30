/**
 * Timing the editor's hot path (docs/roadmap.md, Phase 1b, "Hot path
 * benchmark"): whether Tauri IPC per pencil move keeps up on the M1, or the
 * core has to move into the webview as WASM. Dev builds only (`HOT_PATH` is
 * false in a production build, so all of this drops out).
 *
 * Two things are recorded:
 * - each hot-path call (`applyEdits`, `renderCells`, `cellAt`) from
 *   `invoke` to its answer, in lib/backend.ts;
 * - each pencil move from its pointer event to its cells drawn on the
 *   canvas (queue wait, the edit, the render and `putImageData`), and how
 *   many moves were waiting at once.
 *
 * stylus-core's `examples/stroke_bench.rs` times the same calls without
 * IPC; the difference is what IPC and the webview cost.
 */

export const HOT_PATH = import.meta.env.DEV;

export type HotCall = "applyEdits" | "renderCells" | "cellAt";

const calls: Record<HotCall, number[]> = { applyEdits: [], renderCells: [], cellAt: [] };
const moves: number[] = [];
let waiting = 0;
let mostWaiting = 0;
let firstMove = 0;
let lastDrawn = 0;

export function resetHotPath() {
  for (const list of Object.values(calls)) list.length = 0;
  moves.length = 0;
  waiting = 0;
  mostWaiting = 0;
  firstMove = 0;
  lastDrawn = 0;
}

/** Times one hot-path call. */
export function timeCall<T>(name: HotCall, call: () => Promise<T>): Promise<T> {
  if (!HOT_PATH) return call();
  const start = performance.now();
  return call().finally(() => calls[name].push(performance.now() - start));
}

/** A pencil move's edit was queued; `at` is its pointer event's time. Returns the call for when it's drawn. */
export function moveQueued(at: number): () => void {
  if (!HOT_PATH) return () => undefined;
  if (moves.length === 0 && waiting === 0) firstMove = at;
  waiting += 1;
  mostWaiting = Math.max(mostWaiting, waiting);
  return () => {
    waiting -= 1;
    lastDrawn = performance.now();
    moves.push(lastDrawn - at);
  };
}

export interface Spread {
  count: number;
  median: number;
  p95: number;
  worst: number;
}

function spread(times: number[]): Spread {
  const sorted = [...times].sort((a, b) => a - b);
  const at = (q: number) => sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * q))] ?? 0;
  return { count: sorted.length, median: at(0.5), p95: at(0.95), worst: sorted[sorted.length - 1] ?? 0 };
}

export interface HotPathReport {
  /** Pointer event to cells on the canvas, per move (ms). */
  moves: Spread;
  calls: Record<HotCall, Spread>;
  /** The most moves waiting at once: 1 means none ever waited for another. */
  mostWaiting: number;
  /** Moves drawn per second, first event to last drawn. */
  movesPerSecond: number;
}

export function hotPathReport(): HotPathReport {
  const seconds = (lastDrawn - firstMove) / 1000;
  return {
    moves: spread(moves),
    calls: { applyEdits: spread(calls.applyEdits), renderCells: spread(calls.renderCells), cellAt: spread(calls.cellAt) },
    mostWaiting,
    movesPerSecond: seconds > 0 ? moves.length / seconds : 0,
  };
}

/** The report as lines of text, for the dialog and the console. */
export function hotPathLines(r: HotPathReport): string[] {
  const ms = (s: Spread) => `median ${s.median.toFixed(2)} ms, 95% ${s.p95.toFixed(2)} ms, worst ${s.worst.toFixed(2)} ms (${s.count})`;
  return [
    `Pointer to canvas, per move: ${ms(r.moves)}`,
    `Moves drawn per second: ${r.movesPerSecond.toFixed(0)}; most waiting at once: ${r.mostWaiting}`,
    `applyEdits: ${ms(r.calls.applyEdits)}`,
    `renderCells: ${ms(r.calls.renderCells)}`,
    `cellAt: ${ms(r.calls.cellAt)}`,
  ];
}
