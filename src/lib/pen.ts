/**
 * Input on the art (docs/ui-design.md, "Phones, tablets and pens"): mouse,
 * pen and finger, turned into what the tools understand, so the editor
 * never asks which device drew a stroke.
 *
 * - **A stroke** (`onDown`, `onMove`, `onUp`): the mouse's left or right
 *   button, a pen's tip, or one finger. Right button, a pen's barrel
 *   button and its eraser end are flags on the stroke (`StrokeInput`). The
 *   moves a pen sends between frames (`getCoalescedEvents`) all reach the
 *   tools, so a fast Apple Pencil curve keeps every cell it crossed.
 * - **Hover**: a mouse, or a pen held over the screen (Apple Pencil on the
 *   iPads that sense it, most Windows pens), moves with no stroke, so the
 *   cell under it shows before it touches.
 * - **Fingers**: one finger draws until a pen has drawn; from then on (or
 *   from the start, with iPadOS's "Only Draw with Apple Pencil"), fingers
 *   only move the view, so a hand resting on the screen draws nothing.
 *   Two fingers pan, and pinch steps the zoom; a two-finger tap undoes and
 *   a three-finger tap redoes. A finger's stroke waits `HOLD_MS` before it
 *   starts, so a second finger arriving makes it a gesture, not a dot.
 */

export interface StrokeInput {
  /** The right button, or a pen's barrel button: the tools' second color, or the eraser. */
  alternate: boolean;
  /** A pen's eraser end (Surface Pen, Wacom and others): the Eraser, whatever the tool. */
  eraser: boolean;
  /** Option/Alt held: Pick. */
  pick: boolean;
  /** "mouse", "pen" or "touch". */
  pointerType: string;
  /** The event's time, for the hot-path timing. */
  timeStamp: number;
}

export type Gesture = "undo" | "redo" | "zoomIn" | "zoomOut";

export interface PenTarget<C> {
  /** The cell under a point on the art. */
  cellAt: (event: PointerEvent) => C;
  onDown: (cell: C, input: StrokeInput) => void;
  /** A move during a stroke, or a hover (no stroke). */
  onMove: (cell: C, input: StrokeInput) => void;
  onUp: () => void;
  onGesture?: (gesture: Gesture) => void;
  /** Two fingers moved the view by this many CSS px. */
  onPan: (dx: number, dy: number) => void;
}

/** How long a finger waits for a second one before its stroke starts. */
export const HOLD_MS = 90;
/** Movement under this many CSS px still counts as a tap. */
const TAP_SLOP = 12;
/** A tap of two or three fingers is over within this. */
const TAP_MS = 350;
/** Spreading or closing the fingers by this factor is one zoom step. */
const PINCH_STEP = 1.3;

/** Buttons a pointer event reports (PointerEvent.button / .buttons). */
const BARREL = 2;
const ERASER_BUTTON = 5;
const ERASER_BUTTONS = 32;

/** Fingers draw until a pen has drawn, or the system says pens only. */
let fingersMayDraw = true;

/** From now on only pens draw: a pen has drawn, or iPadOS's "Only Draw with Apple Pencil" is on (src-tauri's pencil.rs). */
export function pensOnly() {
  fingersMayDraw = false;
}

function strokeInput(e: PointerEvent): StrokeInput {
  const pen = e.pointerType === "pen";
  const eraser = pen && (e.button === ERASER_BUTTON || (e.buttons & ERASER_BUTTONS) !== 0);
  return {
    alternate: !eraser && (e.button === BARREL || (e.buttons & BARREL) !== 0),
    eraser,
    pick: e.altKey,
    pointerType: e.pointerType,
    timeStamp: e.timeStamp,
  };
}

interface Touch {
  x: number;
  y: number;
}

interface PendingFinger<C> {
  id: number;
  cell: C;
  input: StrokeInput;
  startX: number;
  startY: number;
  moves: { cell: C; input: StrokeInput }[];
  timer: number;
}

interface TwoFingers {
  /** The fingers' midpoint and spread at the last pan or zoom step. */
  x: number;
  y: number;
  spread: number;
  /** The most fingers down at once, and whether they've moved: for taps. */
  most: number;
  moved: boolean;
  started: number;
}

/** Routes one art canvas's pointer events. `element` gets pointer capture. */
export class PenRouter<C> {
  private stroke: number | null = null;
  private strokeByFinger = false;
  private pending: PendingFinger<C> | null = null;
  private touches = new Map<number, Touch>();
  private gesture: TwoFingers | null = null;

  constructor(private target: PenTarget<C>) {}

  down(e: PointerEvent, element: Element) {
    if (e.pointerType === "touch") {
      this.touches.set(e.pointerId, { x: e.clientX, y: e.clientY });
      element.setPointerCapture(e.pointerId);
      this.fingerDown(e);
      return;
    }
    if (e.pointerType === "mouse" && e.button !== 0 && e.button !== BARREL) return;
    // A pen takes over from a finger (a resting hand, before any pen had
    // drawn); a second pen or mouse while a stroke runs is ignored.
    if (this.stroke !== null && this.strokeByFinger) this.endStroke();
    if (this.stroke !== null) return;
    if (e.pointerType === "pen") pensOnly();
    this.dropPending();
    this.stroke = e.pointerId;
    this.strokeByFinger = false;
    element.setPointerCapture(e.pointerId);
    this.target.onDown(this.target.cellAt(e), strokeInput(e));
  }

  move(e: PointerEvent) {
    if (e.pointerType === "touch") {
      this.fingerMove(e);
      return;
    }
    if (this.stroke !== null && e.pointerId !== this.stroke) return;
    // Every move since the last frame, so a fast pen keeps its path.
    const events = this.stroke !== null && typeof e.getCoalescedEvents === "function" ? e.getCoalescedEvents() : [];
    for (const each of events.length > 0 ? events : [e]) this.target.onMove(this.target.cellAt(each), strokeInput(each));
  }

  up(e: PointerEvent) {
    if (e.pointerType === "touch") {
      this.fingerUp(e);
      return;
    }
    if (e.pointerId === this.stroke) this.endStroke();
  }

  private endStroke() {
    this.stroke = null;
    this.target.onUp();
  }

  // A finger: a stroke (after the hold), or part of a gesture.

  private fingerDown(e: PointerEvent) {
    if (this.touches.size >= 2) {
      // A second finger: a gesture, unless a stroke has already started.
      this.dropPending();
      if (this.stroke === null) this.startGesture();
      else if (this.gesture) this.gesture.most = Math.max(this.gesture.most, this.touches.size);
      return;
    }
    if (this.stroke !== null || !fingersMayDraw) return;
    const input = strokeInput(e);
    const pending: PendingFinger<C> = {
      id: e.pointerId,
      cell: this.target.cellAt(e),
      input,
      startX: e.clientX,
      startY: e.clientY,
      moves: [],
      timer: window.setTimeout(() => this.startFinger(), HOLD_MS),
    };
    this.pending = pending;
  }

  private fingerMove(e: PointerEvent) {
    const touch = this.touches.get(e.pointerId);
    if (!touch) return;
    touch.x = e.clientX;
    touch.y = e.clientY;
    if (this.gesture) {
      this.gestureMove();
      return;
    }
    if (this.pending && this.pending.id === e.pointerId) {
      this.pending.moves.push({ cell: this.target.cellAt(e), input: strokeInput(e) });
      // Moving off at once is a stroke; no need to wait out the hold.
      if (Math.hypot(e.clientX - this.pending.startX, e.clientY - this.pending.startY) > TAP_SLOP) this.startFinger();
      return;
    }
    if (this.stroke === e.pointerId) {
      const events = typeof e.getCoalescedEvents === "function" ? e.getCoalescedEvents() : [];
      for (const each of events.length > 0 ? events : [e]) this.target.onMove(this.target.cellAt(each), strokeInput(each));
    }
  }

  private fingerUp(e: PointerEvent) {
    this.touches.delete(e.pointerId);
    if (this.pending && this.pending.id === e.pointerId) {
      // A quick tap: a dot, or the cursor placed.
      this.startFinger();
    }
    if (this.stroke === e.pointerId) this.endStroke();
    // The fingers seldom lift at the same moment: the gesture ends with the last.
    if (this.gesture && this.touches.size === 0) {
      const g = this.gesture;
      this.gesture = null;
      if (!g.moved && performance.now() - g.started < TAP_MS) {
        if (g.most === 2) this.target.onGesture?.("undo");
        else if (g.most === 3) this.target.onGesture?.("redo");
      }
    }
  }

  private startFinger() {
    const p = this.pending;
    if (!p) return;
    window.clearTimeout(p.timer);
    this.pending = null;
    this.stroke = p.id;
    this.strokeByFinger = true;
    this.target.onDown(p.cell, p.input);
    for (const m of p.moves) this.target.onMove(m.cell, m.input);
    if (!this.touches.has(p.id)) this.endStroke();
  }

  private dropPending() {
    if (!this.pending) return;
    window.clearTimeout(this.pending.timer);
    this.pending = null;
  }

  private fingers() {
    const all = [...this.touches.values()];
    const x = all.reduce((sum, t) => sum + t.x, 0) / all.length;
    const y = all.reduce((sum, t) => sum + t.y, 0) / all.length;
    const spread = all.length < 2 ? 0 : Math.hypot(all[0].x - all[1].x, all[0].y - all[1].y);
    return { x, y, spread };
  }

  private startGesture() {
    const { x, y, spread } = this.fingers();
    this.gesture = { x, y, spread, most: this.touches.size, moved: false, started: performance.now() };
  }

  private gestureMove() {
    const g = this.gesture;
    if (!g || this.touches.size < 2) return;
    g.most = Math.max(g.most, this.touches.size);
    const now = this.fingers();
    const dx = now.x - g.x;
    const dy = now.y - g.y;
    if (!g.moved && Math.hypot(dx, dy) < TAP_SLOP && Math.abs(now.spread - g.spread) < TAP_SLOP) return;
    g.moved = true;
    this.target.onPan(dx, dy);
    g.x = now.x;
    g.y = now.y;
    if (g.spread > 0 && now.spread / g.spread >= PINCH_STEP) {
      this.target.onGesture?.("zoomIn");
      g.spread = now.spread;
    } else if (g.spread > 0 && now.spread / g.spread <= 1 / PINCH_STEP) {
      this.target.onGesture?.("zoomOut");
      g.spread = now.spread;
    }
  }
}
