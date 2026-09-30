/**
 * The Art workspace's editor (docs/ui-design.md, "Art workspace"), Phase 1:
 * the canvas with a text cursor, the tools (Pencil, Eraser, Line,
 * Rectangle, Box, Half-block, Type, Select, Fill, Pick), selection with
 * cut, copy, paste, move, flip, fill and clear, the F-key strip, the
 * Character, Colors and SAUCE panels, and the status bar. Its commands
 * (Edit, Draw, Select, Colors, zoom) are in the menu bar through the
 * command registry (lib/commands.ts), which also runs their shortcuts.
 *
 * Every change goes through stylus-core, one stroke per typed character,
 * drag or command, so undo takes back what the user thinks of as one
 * action. Edits are queued so they reach the core in order, and only the
 * cells each one changed are redrawn. A shape being dragged is redrawn by
 * the core at each move (`drawShape`), so its preview is the art itself;
 * moves that arrive while one is drawing collapse into the latest.
 */
import { useCallback, useEffect, useRef, useState, type KeyboardEvent as ReactKeyboardEvent, type PointerEvent as ReactPointerEvent } from "react";
import { ArtViewer, CellBox, type ArtViewerHandle, type Cell, type Zoom } from "./ArtViewer";
import { SaucePanel } from "./SaucePanel";
import {
  applyEdits,
  cellAt,
  copyCells,
  drawShape,
  floodFill,
  halfBlock,
  pasteCells,
  redo as redoEdit,
  resizeArt,
  selectionOp,
  undo as undoEdit,
  type CellEdit,
  type CellInfo,
  type CellRect,
  type DocumentInfo,
  type EditResult,
  type OpenedArt,
  type Pen,
  type SelectionOp,
  type Shape,
} from "../lib/backend";
import { cp437Char, cp437Code, CP437, FKEY_SETS } from "../lib/cp437";
import type { ActivityUpdate } from "../lib/activity";
import { useCommands, type Handlers } from "../lib/commands";

type RunActivity = <T>(label: string, task: (update: ActivityUpdate) => Promise<T>, key?: string) => Promise<T>;

export type Tool = "pencil" | "eraser" | "line" | "rectangle" | "box" | "halfblock" | "type" | "select" | "fill" | "pick";
type PaintMode = "both" | "color" | "char";
type PanelTab = "character" | "colors" | "sauce";

/** The 16 VGA text-mode colors, for the swatches (the art's colors, not the theme's). */
export const VGA_COLORS = [
  "#000000", "#0000aa", "#00aa00", "#00aaaa", "#aa0000", "#aa00aa", "#aa5500", "#aaaaaa",
  "#555555", "#5555ff", "#55ff55", "#55ffff", "#ff5555", "#ff55ff", "#ffff55", "#ffffff",
];
const COLOR_NAMES = [
  "black", "blue", "green", "cyan", "red", "magenta", "brown", "light gray",
  "dark gray", "light blue", "light green", "light cyan", "light red", "light magenta", "yellow", "white",
];

/** The rail, in docs/ui-design.md's order. Keys work while not typing. */
const TOOLS: { id: Tool; label: string; key: string; hint: string }[] = [
  { id: "pencil", label: "Pencil", key: "P", hint: "Draw the current character and colors. Right-drag erases to the background color" },
  { id: "eraser", label: "Eraser", key: "E", hint: "Erase to spaces in the background color" },
  { id: "line", label: "Line", key: "L", hint: "Drag a straight line of the current character" },
  { id: "rectangle", label: "Rectangle", key: "R", hint: "Drag a rectangle of the current character, outline or filled" },
  { id: "box", label: "Box", key: "B", hint: "Drag a box of line-drawing characters, single or double" },
  { id: "halfblock", label: "Half-block", key: "H", hint: "Paint half-cell pixels in the foreground color; right-drag paints the background color" },
  { id: "type", label: "Type", key: "T", hint: "Click to place the cursor, then type. Esc goes back to the last tool" },
  { id: "select", label: "Select", key: "S", hint: "Drag to select; drag inside the selection to move it. Shift+arrows select from the cursor" },
  { id: "fill", label: "Fill", key: "F", hint: "Fill the cell and every joined cell just like it. Right-click fills with blanks" },
  { id: "pick", label: "Pick", key: "I", hint: "Pick up a cell's character and colors (Option-click in any tool)" },
];

/** The rectangle with corners `a` and `b`, either way round. */
function rectBetween(a: Cell, b: Cell): CellRect {
  return { x: Math.min(a.x, b.x), y: Math.min(a.y, b.y), width: Math.abs(a.x - b.x) + 1, height: Math.abs(a.y - b.y) + 1 };
}

function inside(rect: CellRect, cell: Cell) {
  return cell.x >= rect.x && cell.y >= rect.y && cell.x < rect.x + rect.width && cell.y < rect.y + rect.height;
}

/** A rectangle cut to the canvas, or null when none of it is on it. */
function clampRect(rect: CellRect, info: DocumentInfo): CellRect | null {
  const x = Math.max(rect.x, 0);
  const y = Math.max(rect.y, 0);
  const right = Math.min(rect.x + rect.width, info.columns);
  const bottom = Math.min(rect.y + rect.height, info.rows);
  return right > x && bottom > y ? { x, y, width: right - x, height: bottom - y } : null;
}

const ZOOMS: Zoom[] = [1, 2, 3, 4];

/** A drag in progress on the canvas. */
type Drag =
  | { kind: "paint"; last: Cell; erase: boolean }
  | { kind: "shape"; from: Cell; stroke: number; pen: Pen }
  | { kind: "half"; last: Cell; color: number }
  | { kind: "select"; anchor: Cell; moved: boolean }
  | { kind: "move"; origin: Cell; rect: CellRect; offset: Cell };

interface ArtEditorProps {
  art: OpenedArt;
  /** Called with the document's new info after anything changes it. */
  onInfo: (info: DocumentInfo) => void;
  runActivity: RunActivity;
  onError: (message: string) => void;
  /** Only the visible workspace offers its commands. */
  active: boolean;
}

export function ArtEditor({ art, onInfo, runActivity, onError, active }: ArtEditorProps) {
  const { info } = art;
  const viewer = useRef<ArtViewerHandle>(null);
  const queue = useRef<Promise<unknown>>(Promise.resolve());
  const stroke = useRef(0);
  const drag = useRef<Drag | null>(null);
  /** The latest shape move waiting to be drawn, and whether a draw is running. */
  const shapeJob = useRef<{ next: (() => Promise<EditResult>) | null; running: boolean }>({ next: null, running: false });
  /** Where a Shift+arrow selection started. */
  const anchor = useRef<Cell | null>(null);
  /** The tool Esc goes back to from Type. */
  const lastTool = useRef<Tool>("pencil");
  const infoRef = useRef(info);
  infoRef.current = info;

  const [tool, setTool] = useState<Tool>("type");
  const [paint, setPaint] = useState<PaintMode>("both");
  const [code, setCode] = useState(0xdb);
  const [fg, setFg] = useState(7);
  const [bg, setBg] = useState(0);
  const [fkeySet, setFkeySet] = useState(0);
  const [cursor, setCursor] = useState<Cell>({ x: 0, y: 0 });
  const [hover, setHover] = useState<{ cell: Cell; info: CellInfo | null } | null>(null);
  const [zoom, setZoom] = useState<Zoom>(2);
  const [tab, setTab] = useState<PanelTab>("colors");
  const [filled, setFilled] = useState(false);
  const [double, setDouble] = useState(false);
  const [transparent, setTransparent] = useState(false);
  const [selection, setSelection] = useState<CellRect | null>(null);
  /** How far a selection being dragged has moved, for its outline. */
  const [moveOffset, setMoveOffset] = useState<Cell | null>(null);
  const [clip, setClip] = useState<{ width: number; height: number } | null>(null);

  // A new document starts at the top left.
  useEffect(() => {
    setCursor({ x: 0, y: 0 });
    setHover(null);
    setSelection(null);
    anchor.current = null;
  }, [art.id]);

  // A resize (or undoing one) cuts the selection to the canvas.
  useEffect(() => {
    setSelection((s) => s && clampRect(s, info));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [info.columns, info.rows]);

  function chooseTool(t: Tool) {
    if (t === "type" && tool !== "type") lastTool.current = tool;
    setTool(t);
  }

  // Without iCE, backgrounds are the eight dark colors.
  useEffect(() => {
    if (!info.settings.iceColors && bg > 7) setBg(bg - 8);
  }, [info.settings.iceColors, bg]);

  /** Runs `task` after every earlier edit, in order. */
  const enqueue = useCallback(<T,>(task: () => Promise<T>): Promise<T> => {
    const next = queue.current.then(task, task);
    queue.current = next.catch(() => undefined);
    return next;
  }, []);

  const afterEdit = useCallback(
    async (result: EditResult) => {
      if (result.info) onInfo(result.info);
      else if (result.canUndo !== infoRef.current.canUndo || result.canRedo !== infoRef.current.canRedo || result.edited !== infoRef.current.edited) {
        onInfo({ ...infoRef.current, canUndo: result.canUndo, canRedo: result.canRedo, edited: result.edited });
      }
      if (result.dirty && !result.info) await viewer.current?.redraw(result.dirty);
    },
    [onInfo],
  );

  const edit = useCallback(
    (edits: CellEdit[], strokeId: number) =>
      enqueue(async () => {
        const result = await applyEdits(art.id, strokeId, edits);
        await afterEdit(result);
      }).catch((e) => onError(`Couldn't change the art: ${e}`)),
    [art.id, enqueue, afterEdit, onError],
  );

  const undo = useCallback(
    () => enqueue(async () => afterEdit(await undoEdit(art.id))).catch((e) => onError(`Couldn't undo: ${e}`)),
    [art.id, enqueue, afterEdit, onError],
  );
  const redo = useCallback(
    () => enqueue(async () => afterEdit(await redoEdit(art.id))).catch((e) => onError(`Couldn't redo: ${e}`)),
    [art.id, enqueue, afterEdit, onError],
  );

  /** Runs one core call after every earlier edit, and shows what it changed. */
  const run = useCallback(
    (what: string, call: () => Promise<EditResult>) =>
      enqueue(async () => afterEdit(await call())).catch((e) => onError(`Couldn't ${what}: ${e}`)),
    [enqueue, afterEdit, onError],
  );

  /**
   * Draws the latest of a drag's moves. A move that comes while one is
   * drawing replaces any other waiting, so a fast drag never falls behind.
   */
  function requestDraw(call: () => Promise<EditResult>) {
    const job = shapeJob.current;
    job.next = call;
    if (job.running) return;
    job.running = true;
    void enqueue(async () => {
      try {
        while (job.next) {
          const next = job.next;
          job.next = null;
          await afterEdit(await next());
        }
      } finally {
        job.running = false;
      }
    }).catch((e) => onError(`Couldn't draw: ${e}`));
  }

  /** What the current paint mode puts down. */
  const pen: Pen = {
    code: paint === "color" ? undefined : code,
    fg: paint === "char" ? undefined : fg,
    bg: paint === "char" ? undefined : bg,
  };
  const eraserPen: Pen = { code: 32, fg, bg };

  /** The edit the current tool and paint mode make at a cell. */
  function paintEdit(cell: Cell, erase: boolean): CellEdit {
    return { x: cell.x, y: cell.y, ...(erase ? eraserPen : pen) };
  }

  function shapeFor(from: Cell, to: Cell): Shape {
    const a = { x: from.x, y: from.y };
    const b = { x: to.x, y: to.y };
    if (tool === "rectangle") return { kind: "rectangle", from: a, to: b, filled };
    if (tool === "box") return { kind: "box", from: a, to: b, double };
    return { kind: "line", from: a, to: b };
  }

  function onSelection(what: string, op: SelectionOp) {
    if (!selection) return;
    const rect = selection;
    void run(what, () => selectionOp(art.id, ++stroke.current, rect, op));
  }

  async function copy(): Promise<boolean> {
    if (!selection) return false;
    try {
      const size = await enqueue(() => copyCells(art.id, selection));
      if (size) setClip(size);
      return !!size;
    } catch (e) {
      onError(`Couldn't copy: ${e}`);
      return false;
    }
  }

  async function cut() {
    if (await copy()) onSelection("cut", { kind: "clear" });
  }

  /** Pastes at the selection's top left (or the cursor), and selects what was pasted. */
  function paste(asTransparent: boolean) {
    if (!clip) return;
    const at = selection ? { x: selection.x, y: selection.y } : { x: cursor.x, y: cursor.y };
    void enqueue(async () => {
      const [result, placed] = await pasteCells(art.id, ++stroke.current, at, asTransparent);
      await afterEdit(result);
      if (placed) setSelection(clampRect(placed, infoRef.current));
    }).catch((e) => onError(`Couldn't paste: ${e}`));
  }

  function selectAll() {
    setSelection({ x: 0, y: 0, width: info.columns, height: info.rows });
    anchor.current = null;
  }

  function deselect() {
    setSelection(null);
    anchor.current = null;
  }

  async function pick(cell: Cell) {
    try {
      const c = await cellAt(art.id, cell.x, cell.y);
      if (!c) return;
      if (c.code < 256) setCode(c.code);
      setFg(c.fg);
      setBg(infoRef.current.settings.iceColors ? c.bg : c.bg & 7);
    } catch (e) {
      onError(String(e));
    }
  }

  /** Every cell on the line from `a` to `b` (Bresenham), so a fast drag leaves no gaps. */
  function lineCells(a: Cell, b: Cell): Cell[] {
    const cells: Cell[] = [];
    let { x, y } = a;
    const dx = Math.abs(b.x - x);
    const dy = -Math.abs(b.y - y);
    const sx = x < b.x ? 1 : -1;
    const sy = y < b.y ? 1 : -1;
    let err = dx + dy;
    for (;;) {
      cells.push({ x, y });
      if (x === b.x && y === b.y) break;
      const e2 = 2 * err;
      if (e2 >= dy) {
        err += dy;
        x += sx;
      }
      if (e2 <= dx) {
        err += dx;
        y += sy;
      }
    }
    return cells;
  }

  const moveCursor = useCallback((cell: Cell) => {
    const i = infoRef.current;
    const next = { x: Math.min(Math.max(cell.x, 0), i.columns - 1), y: Math.min(Math.max(cell.y, 0), i.rows - 1) };
    setCursor(next);
    viewer.current?.reveal(next);
  }, []);

  /** Types one character code at the cursor and moves right. */
  function typeCode(c: number) {
    const at = cursor;
    void edit([{ x: at.x, y: at.y, code: c, fg: paint === "char" ? undefined : fg, bg: paint === "char" ? undefined : bg }], ++stroke.current);
    moveCursor({ x: at.x + 1, y: at.y });
  }

  /** Moves down a row, adding one at the bottom when the cursor is on the last. */
  async function downOrGrow(x: number) {
    const i = infoRef.current;
    if (cursor.y + 1 < i.rows) {
      moveCursor({ x, y: cursor.y + 1 });
      return;
    }
    try {
      const grown = await enqueue(() => resizeArt(art.id, i.columns, i.rows + 1));
      onInfo(grown);
      setCursor({ x, y: cursor.y + 1 });
    } catch (e) {
      onError(String(e));
    }
  }

  function onCanvasKey(e: ReactKeyboardEvent<HTMLDivElement>) {
    if (e.metaKey || e.ctrlKey) {
      if (e.ctrlKey && (e.key === "ArrowLeft" || e.key === "ArrowRight")) {
        e.preventDefault();
        setFkeySet((s) => (s + (e.key === "ArrowRight" ? 1 : FKEY_SETS.length - 1)) % FKEY_SETS.length);
      }
      return;
    }
    const page = 20;
    // Shift+arrows select from the cursor.
    const step: Record<string, Cell> = { ArrowLeft: { x: -1, y: 0 }, ArrowRight: { x: 1, y: 0 }, ArrowUp: { x: 0, y: -1 }, ArrowDown: { x: 0, y: 1 } };
    if (e.shiftKey && step[e.key]) {
      e.preventDefault();
      const from = selection && anchor.current ? anchor.current : cursor;
      anchor.current = from;
      const next = {
        x: Math.min(Math.max(cursor.x + step[e.key].x, 0), info.columns - 1),
        y: Math.min(Math.max(cursor.y + step[e.key].y, 0), info.rows - 1),
      };
      moveCursor(next);
      setSelection(rectBetween(from, next));
      return;
    }
    if (e.key === "Escape") {
      e.preventDefault();
      if (selection) deselect();
      else if (tool === "type") setTool(lastTool.current);
      return;
    }
    if (selection && (e.key === "Delete" || e.key === "Backspace")) {
      e.preventDefault();
      onSelection("clear the selection", { kind: "clear" });
      return;
    }
    // Tool keys and X (swap colors), only while not typing, so typing
    // never switches tools.
    if (tool !== "type" && e.key.length === 1 && !e.altKey) {
      const t = TOOLS.find((t) => t.key === e.key.toUpperCase());
      if (t) {
        e.preventDefault();
        chooseTool(t.id);
      } else if (e.key.toLowerCase() === "x") {
        e.preventDefault();
        swapColors();
      }
      return;
    }
    const fkey = /^F(\d+)$/.exec(e.key);
    if (fkey && Number(fkey[1]) >= 1 && Number(fkey[1]) <= 10) {
      e.preventDefault();
      typeCode(FKEY_SETS[fkeySet][Number(fkey[1]) - 1]);
      return;
    }
    switch (e.key) {
      case "ArrowLeft":
        e.preventDefault();
        moveCursor({ x: cursor.x - 1, y: cursor.y });
        return;
      case "ArrowRight":
        e.preventDefault();
        moveCursor({ x: cursor.x + 1, y: cursor.y });
        return;
      case "ArrowUp":
        e.preventDefault();
        moveCursor({ x: cursor.x, y: cursor.y - 1 });
        return;
      case "ArrowDown":
        e.preventDefault();
        void downOrGrow(cursor.x);
        return;
      case "Home":
        e.preventDefault();
        moveCursor({ x: 0, y: cursor.y });
        return;
      case "End":
        e.preventDefault();
        moveCursor({ x: info.columns - 1, y: cursor.y });
        return;
      case "PageUp":
        e.preventDefault();
        moveCursor({ x: cursor.x, y: cursor.y - page });
        return;
      case "PageDown":
        e.preventDefault();
        moveCursor({ x: cursor.x, y: cursor.y + page });
        return;
      case "Enter":
        e.preventDefault();
        void downOrGrow(0);
        return;
      case "Backspace":
        e.preventDefault();
        if (cursor.x > 0) {
          const at = { x: cursor.x - 1, y: cursor.y };
          void edit([{ x: at.x, y: at.y, code: 32, fg, bg }], ++stroke.current);
          moveCursor(at);
        }
        return;
      case "Delete":
        e.preventDefault();
        void edit([{ x: cursor.x, y: cursor.y, code: 32, fg, bg }], ++stroke.current);
        return;
    }
    if (e.key.length === 1 && !e.altKey) {
      const c = cp437Code(e.key);
      if (c !== null) {
        e.preventDefault();
        typeCode(c);
      }
    }
  }

  function swapColors() {
    if (fg >= (info.settings.iceColors ? 16 : 8)) return;
    setFg(bg);
    setBg(fg);
  }

  function zoomBy(step: number) {
    const now = zoom === "fit" ? (step > 0 ? 1 : 2) : zoom;
    setZoom(ZOOMS[Math.min(Math.max(ZOOMS.indexOf(now) + step, 0), ZOOMS.length - 1)]);
  }

  // The menu bar's commands (lib/commands.ts), while this workspace shows.
  const has = !!selection;
  const handlers: Handlers = {
    "edit.undo": { run: () => void undo(), enabled: info.canUndo },
    "edit.redo": { run: () => void redo(), enabled: info.canRedo },
    "edit.cut": { run: () => void cut(), enabled: has },
    "edit.copy": { run: () => void copy(), enabled: has },
    "edit.paste": { run: () => paste(transparent), enabled: !!clip },
    "edit.pasteTransparent": { run: () => paste(true), enabled: !!clip },
    "edit.delete": { run: () => onSelection("clear the selection", { kind: "clear" }), enabled: has },
    "edit.selectAll": { run: selectAll },
    "select.all": { run: selectAll },
    "select.none": { run: deselect, enabled: has },
    "select.flipHorizontal": { run: () => onSelection("flip the selection", { kind: "flipHorizontal" }), enabled: has },
    "select.flipVertical": { run: () => onSelection("flip the selection", { kind: "flipVertical" }), enabled: has },
    "select.fill": { run: () => onSelection("fill the selection", { kind: "fill", pen }), enabled: has },
    "select.clear": { run: () => onSelection("clear the selection", { kind: "clear" }), enabled: has },
    "draw.filled": { run: () => setFilled((v) => !v), checked: filled },
    "draw.double": { run: () => setDouble((v) => !v), checked: double },
    "colors.swap": { run: swapColors, enabled: fg < (info.settings.iceColors ? 16 : 8) },
    "colors.paintBoth": { run: () => setPaint("both"), checked: paint === "both" },
    "colors.paintColor": { run: () => setPaint("color"), checked: paint === "color" },
    "colors.paintChar": { run: () => setPaint("char"), checked: paint === "char" },
    "view.zoomIn": { run: () => zoomBy(1), enabled: zoom !== 4 },
    "view.zoomOut": { run: () => zoomBy(-1), enabled: zoom !== 1 },
    "view.actualSize": { run: () => setZoom(1), checked: zoom === 1 },
    "view.fit": { run: () => setZoom("fit"), checked: zoom === "fit" },
  };
  for (const t of TOOLS) handlers[`tool.${t.id}`] = { run: () => chooseTool(t.id), checked: tool === t.id };
  useCommands("art-editor", active ? handlers : {});

  const pointer = {
    onCellDown(cell: Cell, e: ReactPointerEvent<HTMLDivElement>) {
      (e.currentTarget.closest(".art-editor-canvas") as HTMLElement | null)?.focus();
      const right = e.button === 2;
      if (tool === "pick" || e.altKey) {
        void pick(cell);
        return;
      }
      switch (tool) {
        case "type":
          moveCursor(cell);
          deselect();
          return;
        case "pencil":
        case "eraser": {
          const erase = tool === "eraser" || right;
          drag.current = { kind: "paint", last: cell, erase };
          stroke.current += 1;
          void edit([paintEdit(cell, erase)], stroke.current);
          return;
        }
        case "line":
        case "rectangle":
        case "box": {
          const id = ++stroke.current;
          const shapePen = right && tool !== "box" ? eraserPen : pen;
          drag.current = { kind: "shape", from: cell, stroke: id, pen: shapePen };
          const shape = shapeFor(cell, cell);
          requestDraw(() => drawShape(art.id, id, shape, shapePen));
          return;
        }
        case "halfblock": {
          const color = right ? bg : fg;
          const at = { x: cell.x, y: cell.half ?? cell.y * 2 };
          drag.current = { kind: "half", last: at, color };
          stroke.current += 1;
          const id = stroke.current;
          void run("draw", () => halfBlock(art.id, id, at, at, color));
          return;
        }
        case "fill": {
          const id = ++stroke.current;
          const fillPen = right ? eraserPen : pen;
          void run("fill", () => floodFill(art.id, id, { x: cell.x, y: cell.y }, fillPen));
          return;
        }
        case "select":
          if (selection && inside(selection, cell)) {
            drag.current = { kind: "move", origin: cell, rect: selection, offset: { x: 0, y: 0 } };
            setMoveOffset({ x: 0, y: 0 });
          } else {
            drag.current = { kind: "select", anchor: cell, moved: false };
            anchor.current = cell;
            setSelection(rectBetween(cell, cell));
          }
          return;
      }
    },
    onCellMove(cell: Cell) {
      if (!hover || hover.cell.x !== cell.x || hover.cell.y !== cell.y) {
        setHover({ cell, info: null });
        void cellAt(art.id, cell.x, cell.y).then(
          (c) => setHover((h) => (h && h.cell.x === cell.x && h.cell.y === cell.y ? { cell, info: c } : h)),
          () => undefined,
        );
      }
      const d = drag.current;
      if (!d) return;
      switch (d.kind) {
        case "paint": {
          if (d.last.x === cell.x && d.last.y === cell.y) return;
          const cells = lineCells(d.last, cell).slice(1);
          d.last = cell;
          void edit(
            cells.map((c) => paintEdit(c, d.erase)),
            stroke.current,
          );
          return;
        }
        case "shape": {
          const shape = shapeFor(d.from, cell);
          requestDraw(() => drawShape(art.id, d.stroke, shape, d.pen));
          return;
        }
        case "half": {
          const at = { x: cell.x, y: cell.half ?? cell.y * 2 };
          if (at.x === d.last.x && at.y === d.last.y) return;
          const from = d.last;
          d.last = at;
          const id = stroke.current;
          void run("draw", () => halfBlock(art.id, id, from, at, d.color));
          return;
        }
        case "select":
          if (cell.x !== d.anchor.x || cell.y !== d.anchor.y) d.moved = true;
          setSelection(rectBetween(d.anchor, cell));
          moveCursor(cell);
          return;
        case "move":
          d.offset = { x: cell.x - d.origin.x, y: cell.y - d.origin.y };
          setMoveOffset(d.offset);
          return;
      }
    },
    onCellUp() {
      const d = drag.current;
      drag.current = null;
      if (!d) return;
      if (d.kind === "select" && !d.moved) {
        // A click without a drag places the cursor instead.
        deselect();
        moveCursor(d.anchor);
      } else if (d.kind === "move") {
        setMoveOffset(null);
        if (d.offset.x === 0 && d.offset.y === 0) return;
        const to = { x: d.rect.x + d.offset.x, y: d.rect.y + d.offset.y };
        const rect = d.rect;
        void run("move the selection", () => selectionOp(art.id, ++stroke.current, rect, { kind: "move", to }));
        setSelection(clampRect({ ...rect, ...to }, info));
      }
    },
    onLeave() {
      setHover(null);
    },
  };

  const bgCount = info.settings.iceColors ? 16 : 8;
  const flags = [info.format, info.settings.iceColors ? "iCE" : "blink", info.settings.letterSpacing ? "9-px" : "8-px"].join(" · ");

  return (
    <div className="art-editor" data-testid="art-editor">
      <div className="editor-layout">
        <nav className="tool-rail" aria-label="Tools">
          {TOOLS.map((t) => (
            <button
              key={t.id}
              type="button"
              className={`tool-btn${tool === t.id ? " selected" : ""}`}
              aria-pressed={tool === t.id}
              data-testid={`tool-${t.id}`}
              title={`${t.hint} (${t.key})`}
              onClick={() => chooseTool(t.id)}
            >
              {t.label}
            </button>
          ))}
          <label className="tool-paint" title="What the tools and typing change">
            <span>Paints</span>
            <select data-testid="paint-mode" value={paint} onChange={(e) => setPaint(e.currentTarget.value as PaintMode)}>
              <option value="both">Character and colors</option>
              <option value="color">Colors only</option>
              <option value="char">Character only</option>
            </select>
          </label>
          {tool === "rectangle" && (
            <label className="tool-option">
              <input type="checkbox" data-testid="option-filled" checked={filled} onChange={(e) => setFilled(e.currentTarget.checked)} />
              <span>Filled</span>
            </label>
          )}
          {tool === "box" && (
            <label className="tool-option">
              <input type="checkbox" data-testid="option-double" checked={double} onChange={(e) => setDouble(e.currentTarget.checked)} />
              <span>Double lines</span>
            </label>
          )}
          <div className="rail-section" data-testid="selection-actions" aria-label="Selection">
            <span className="rail-heading">Selection</span>
            <button type="button" className="small" disabled={!selection} onClick={() => void cut()} title="Cut (⌘X)">
              Cut
            </button>
            <button type="button" className="small" disabled={!selection} onClick={() => void copy()} title="Copy (⌘C)">
              Copy
            </button>
            <button
              type="button"
              className="small"
              data-testid="paste-button"
              disabled={!clip}
              onClick={() => paste(transparent)}
              title="Paste at the selection or the cursor (⌘V; ⌥⌘V pastes transparent)"
            >
              Paste
            </button>
            <label className="tool-option" title="Pasted blanks on black let the art show through">
              <input type="checkbox" data-testid="option-transparent" checked={transparent} onChange={(e) => setTransparent(e.currentTarget.checked)} />
              <span>Transparent</span>
            </label>
            <button type="button" className="small" disabled={!selection} onClick={() => onSelection("flip the selection", { kind: "flipHorizontal" })} title="Flip left to right">
              Flip ↔
            </button>
            <button type="button" className="small" disabled={!selection} onClick={() => onSelection("flip the selection", { kind: "flipVertical" })} title="Flip top to bottom">
              Flip ↕
            </button>
            <button type="button" className="small" disabled={!selection} onClick={() => onSelection("fill the selection", { kind: "fill", pen })} title="Fill with the current character and colors">
              Fill
            </button>
            <button type="button" className="small" disabled={!selection} onClick={() => onSelection("clear the selection", { kind: "clear" })} title="Clear to blanks (Delete)">
              Clear
            </button>
          </div>
        </nav>

        <div
          className={`art-editor-canvas tool-${tool}`}
          tabIndex={0}
          role="application"
          aria-label={`${art.name}: arrows move the cursor, typing writes at it`}
          data-testid="art-canvas"
          onKeyDown={onCanvasKey}
        >
          <ArtViewer
            ref={viewer}
            id={art.id}
            name={art.name}
            info={info}
            zoom={zoom}
            runActivity={runActivity}
            onError={onError}
            pointer={pointer}
            overlay={
              <>
                <CellBox info={info} rect={{ x: cursor.x, y: cursor.y, width: 1, height: 1 }} className="text-cursor" />
                {selection && (
                  <CellBox
                    info={info}
                    rect={moveOffset ? { ...selection, x: selection.x + moveOffset.x, y: selection.y + moveOffset.y } : selection}
                    className="selection-box"
                  />
                )}
              </>
            }
          />
        </div>

        <aside className="panel context-panel" data-testid="context-panel">
          <div className="panel-tabs" role="tablist">
            {(["character", "colors", "sauce"] as PanelTab[]).map((t) => (
              <button
                key={t}
                type="button"
                role="tab"
                aria-selected={tab === t}
                className={`panel-tab${tab === t ? " selected" : ""}`}
                data-testid={`panel-tab-${t}`}
                onClick={() => setTab(t)}
              >
                {t === "character" ? "Character" : t === "colors" ? "Colors" : "SAUCE"}
              </button>
            ))}
          </div>
          {tab === "character" && (
            <div className="char-grid" role="listbox" aria-label="Characters" data-testid="char-grid">
              {CP437.map((ch, c) => (
                <button
                  key={c}
                  type="button"
                  role="option"
                  aria-selected={code === c}
                  className={`char-cell${code === c ? " selected" : ""}`}
                  title={`${c}`}
                  onClick={() => setCode(c)}
                >
                  {c === 0 || c === 32 || c === 255 ? " " : ch}
                </button>
              ))}
            </div>
          )}
          {tab === "colors" && (
            <div className="color-panel" data-testid="color-panel">
              <p className="desc">Click for the foreground, right-click (or Shift-click) for the background.</p>
              <div className="swatches">
                {VGA_COLORS.map((color, i) => (
                  <button
                    key={color}
                    type="button"
                    className={`swatch${fg === i ? " is-fg" : ""}${bg === i ? " is-bg" : ""}`}
                    style={{ background: color }}
                    aria-label={`${COLOR_NAMES[i]}${fg === i ? ", foreground" : ""}${bg === i ? ", background" : ""}`}
                    data-testid={`swatch-${i}`}
                    onClick={(e) => (e.shiftKey ? i < bgCount && setBg(i) : setFg(i))}
                    onContextMenu={(e) => {
                      e.preventDefault();
                      if (i < bgCount) setBg(i);
                    }}
                  >
                    {(fg === i || bg === i) && (
                      <span className="swatch-mark" aria-hidden="true">
                        {fg === i ? "F" : ""}
                        {bg === i ? "B" : ""}
                      </span>
                    )}
                  </button>
                ))}
              </div>
              {!info.settings.iceColors && <p className="desc">iCE colors are off, so backgrounds are the first eight colors.</p>}
              <button
                type="button"
                className="small"
                disabled={fg >= bgCount}
                title={fg >= bgCount ? "Without iCE colors, a bright foreground can't be a background" : "Swap foreground and background (X)"}
                onClick={swapColors}
              >
                Swap colors
              </button>
            </div>
          )}
          {tab === "sauce" && <SaucePanel name={art.name} info={info} embedded />}
        </aside>
      </div>

      <div className="fkey-strip" data-testid="fkey-strip">
        {FKEY_SETS[fkeySet].map((c, i) => (
          <button key={i} type="button" className="fkey" title={`F${i + 1}: character ${c}`} onClick={() => typeCode(c)}>
            <span className="fkey-name">F{i + 1}</span>
            <span className="fkey-char">{cp437Char(c)}</span>
          </button>
        ))}
        <button type="button" className="small fkey-set" title="Previous set (⌃←)" onClick={() => setFkeySet((s) => (s + FKEY_SETS.length - 1) % FKEY_SETS.length)}>
          ◄
        </button>
        <span className="fkey-set-label">
          Set {fkeySet + 1}/{FKEY_SETS.length}
        </span>
        <button type="button" className="small fkey-set" title="Next set (⌃→)" onClick={() => setFkeySet((s) => (s + 1) % FKEY_SETS.length)}>
          ►
        </button>
        <span className="current-pen" aria-label={`Current: character ${code}, ${COLOR_NAMES[fg]} on ${COLOR_NAMES[bg]}`}>
          <span className="pen-sample" style={{ color: VGA_COLORS[fg], background: VGA_COLORS[bg] }}>
            {cp437Char(code)}
          </span>
        </span>
      </div>

      <div className="status-bar" data-testid="status-bar" aria-live="off">
        <span className="doc-name" data-testid="art-name">
          {art.name}
          {info.edited && <span className="edited-mark" aria-label="unsaved changes"> •</span>}
        </span>
        <span>{TOOLS.find((t) => t.id === tool)?.label}</span>
        <span>
          {cp437Char(code)} {code}
        </span>
        <span>
          FG {fg} BG {bg}
        </span>
        <span data-testid="status-cursor">
          {cursor.x + 1},{cursor.y + 1}
        </span>
        {selection && (
          <span data-testid="status-selection">
            selection {selection.width}×{selection.height}
          </span>
        )}
        {hover && (
          <span>
            under pointer {hover.cell.x + 1},{hover.cell.y + 1}
            {hover.info ? `: ${hover.info.code} FG ${hover.info.fg} BG ${hover.info.bg}${hover.info.truecolor ? " (24-bit)" : ""}` : ""}
          </span>
        )}
        <span>
          {info.columns}×{info.rows}
        </span>
        <span>{flags}</span>
        <span>{zoom === "fit" ? "Fit" : `${zoom * 100}%`}</span>
        {info.edited && <span data-testid="status-edited">Edited</span>}
      </div>
    </div>
  );
}
