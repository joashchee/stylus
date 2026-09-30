/**
 * The command registry (docs/ui-design.md, rule 4): every command's name,
 * shortcut and menu placement, in one table. The menu bar, the shortcuts
 * and Help → Keyboard shortcuts all read it, so a command is added once
 * (and the command palette reads it in Phase 3).
 *
 * The table says what a command is; the component that can do it says
 * whether it can right now. `useCommands` hands the registry a component's
 * handlers (run, enabled, checked) on every render. A command with no
 * handler shows disabled, except in a `group`, whose items show only while
 * someone provides them (the Amiga fonts for a text file), and a `submenu`,
 * whose items are whatever is provided under its id (Open Recent's files)
 * and which shows disabled while none can run.
 *
 * A menu only lists what its phase has shipped: no greyed-out placeholders
 * for features still to come.
 */
import { createContext, createElement, useContext, useEffect, useLayoutEffect, useMemo, useRef, useState, type ReactNode } from "react";

export type MenuId = "file" | "edit" | "draw" | "select" | "colors" | "view" | "help";

export interface CommandDef {
  id: string;
  label: string;
  /**
   * `mod` is ⌘ on the Mac and Ctrl elsewhere; then `shift`, `alt`, and a
   * key, joined by `+` (e.g. `mod+shift+z`). Keys are lower case.
   */
  shortcut?: string;
  /** More keys for the same command, not shown in menus (⌘Y for Redo). */
  alsoKeys?: string[];
  /**
   * The shortcut belongs to the canvas: it works only while the art has the
   * keyboard and isn't being typed on (tool letters, Delete, Esc), so the
   * canvas handles it and the registry only shows it.
   */
  canvasKey?: boolean;
}

export type MenuEntry = CommandDef | "-" | { group: string } | { submenu: string; label: string };

export interface MenuDef {
  id: MenuId;
  label: string;
  /** The access letter (Alt/Option + it opens the menu), a letter of the label. */
  access: string;
  items: MenuEntry[];
}

/** Every Phase 1 command, by menu, in menu order. */
export const MENUS: MenuDef[] = [
  {
    id: "file",
    label: "File",
    access: "f",
    items: [
      { id: "file.new", label: "New…", shortcut: "mod+n" },
      { id: "file.open", label: "Open…", shortcut: "mod+o" },
      { submenu: "file.recent", label: "Open Recent" },
      { group: "file.recover" },
      "-",
      { id: "file.save", label: "Save", shortcut: "mod+s" },
      { id: "file.saveAs", label: "Save As…", shortcut: "mod+shift+s" },
      "-",
      { id: "file.exportPng", label: "Export PNG…", shortcut: "mod+shift+e" },
    ],
  },
  {
    id: "edit",
    label: "Edit",
    access: "e",
    items: [
      { id: "edit.undo", label: "Undo", shortcut: "mod+z" },
      { id: "edit.redo", label: "Redo", shortcut: "mod+shift+z", alsoKeys: ["mod+y"] },
      "-",
      { id: "edit.cut", label: "Cut", shortcut: "mod+x" },
      { id: "edit.copy", label: "Copy", shortcut: "mod+c" },
      { id: "edit.paste", label: "Paste", shortcut: "mod+v" },
      { id: "edit.pasteTransparent", label: "Paste Transparent", shortcut: "mod+alt+v" },
      { id: "edit.delete", label: "Delete", shortcut: "delete", canvasKey: true },
      "-",
      { id: "edit.selectAll", label: "Select All", shortcut: "mod+a" },
    ],
  },
  {
    id: "draw",
    label: "Draw",
    access: "d",
    items: [
      { id: "tool.pencil", label: "Pencil", shortcut: "p", canvasKey: true },
      { id: "tool.eraser", label: "Eraser", shortcut: "e", canvasKey: true },
      { id: "tool.line", label: "Line", shortcut: "l", canvasKey: true },
      { id: "tool.rectangle", label: "Rectangle", shortcut: "r", canvasKey: true },
      { id: "tool.box", label: "Box", shortcut: "b", canvasKey: true },
      { id: "tool.halfblock", label: "Half-block Brush", shortcut: "h", canvasKey: true },
      { id: "tool.type", label: "Type", shortcut: "t", canvasKey: true },
      { id: "tool.select", label: "Select", shortcut: "s", canvasKey: true },
      { id: "tool.fill", label: "Fill", shortcut: "f", canvasKey: true },
      { id: "tool.pick", label: "Pick", shortcut: "i", canvasKey: true },
      "-",
      { id: "draw.filled", label: "Filled Rectangles" },
      { id: "draw.double", label: "Double Box Lines" },
    ],
  },
  {
    id: "select",
    label: "Select",
    access: "s",
    items: [
      { id: "select.all", label: "Select All", shortcut: "mod+a" },
      { id: "select.none", label: "Deselect", shortcut: "escape", canvasKey: true },
      "-",
      { id: "select.flipHorizontal", label: "Flip Horizontal" },
      { id: "select.flipVertical", label: "Flip Vertical" },
      "-",
      { id: "select.fill", label: "Fill Selection" },
      { id: "select.clear", label: "Clear", shortcut: "delete", canvasKey: true },
    ],
  },
  {
    id: "colors",
    label: "Colors",
    access: "c",
    items: [
      { id: "colors.swap", label: "Swap Foreground and Background", shortcut: "x", canvasKey: true },
      { id: "colors.ice", label: "iCE Colors" },
      "-",
      { id: "colors.paintBoth", label: "Paint Character and Colors" },
      { id: "colors.paintColor", label: "Paint Colors Only" },
      { id: "colors.paintChar", label: "Paint Character Only" },
    ],
  },
  {
    id: "view",
    label: "View",
    access: "v",
    items: [
      { id: "view.zoomIn", label: "Zoom In", shortcut: "mod+=" },
      { id: "view.zoomOut", label: "Zoom Out", shortcut: "mod+-" },
      { id: "view.actualSize", label: "Actual Size", shortcut: "mod+0" },
      { id: "view.fit", label: "Fit Width" },
      "-",
      { id: "view.letterSpacing", label: "9-px Spacing" },
      { id: "view.aspect", label: "Aspect Ratio" },
      { group: "view.textFont" },
      "-",
      { id: "view.contrastLint", label: "Check Contrast" },
    ],
  },
  {
    id: "help",
    label: "Help",
    access: "h",
    items: [
      { id: "help.shortcuts", label: "Keyboard Shortcuts", shortcut: "mod+/" },
      { id: "help.about", label: "About Stylus" },
      // Development tools (lib/hotPath.ts); nothing provides them in a release build.
      { group: "help.dev" },
    ],
  },
];

/** What a component can do with a command right now. */
export interface Handler {
  run: () => void;
  /** Default true. */
  enabled?: boolean;
  /** A tick in the menu (a tool, a setting that's on). */
  checked?: boolean;
  /** For a group's or submenu's items, which the table doesn't name. */
  label?: string;
  /** In a submenu, a separator goes above this item. */
  separatorBefore?: boolean;
}

export type Handlers = Record<string, Handler>;

/** A menu item as it stands now, for the in-window and native menus. */
export interface MenuItem {
  id: string;
  label: string;
  shortcut?: string;
  handler?: Handler;
  /** A submenu's items. */
  children?: (MenuItem | "-")[];
}

/** An item that can be chosen now: a command that can run, or a submenu with one that can. */
export function itemEnabled(item: MenuItem | "-"): item is MenuItem {
  if (item === "-") return false;
  if (item.children) return item.children.some(itemEnabled);
  return !!item.handler && item.handler.enabled !== false;
}

/** A menu's items as they stand now: groups filled in, stray separators dropped. */
export function menuItems(menu: MenuDef, handlers: Handlers): (MenuItem | "-")[] {
  const out: (MenuItem | "-")[] = [];
  for (const entry of menu.items) {
    if (entry === "-") {
      out.push("-");
    } else if ("group" in entry) {
      const prefix = `${entry.group}.`;
      const ids = Object.keys(handlers).filter((id) => id.startsWith(prefix));
      if (ids.length > 0) out.push("-");
      for (const id of ids) out.push({ id, label: handlers[id].label ?? id.slice(prefix.length), handler: handlers[id] });
    } else if ("submenu" in entry) {
      const prefix = `${entry.submenu}.`;
      const children: (MenuItem | "-")[] = [];
      for (const id of Object.keys(handlers).filter((id) => id.startsWith(prefix))) {
        if (handlers[id].separatorBefore && children.length > 0) children.push("-");
        children.push({ id, label: handlers[id].label ?? id.slice(prefix.length), handler: handlers[id] });
      }
      out.push({ id: entry.submenu, label: entry.label, children });
    } else {
      out.push({ id: entry.id, label: entry.label, shortcut: entry.shortcut, handler: handlers[entry.id] });
    }
  }
  return out.filter((item, i) => item !== "-" || (i > 0 && i < out.length - 1 && out[i - 1] !== "-"));
}

export const IS_MAC = /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);

/** A shortcut as the platform writes it: ⇧⌘S on the Mac, Ctrl+Shift+S elsewhere. */
export function shortcutLabel(shortcut: string): string {
  const parts = shortcut.split("+");
  const key = parts[parts.length - 1] || "+";
  const names: Record<string, string> = { delete: IS_MAC ? "⌫" : "Delete", escape: "Esc", "=": "+", "-": "−" };
  const keyName = names[key] ?? key.toUpperCase();
  const mods = parts.slice(0, -1);
  if (IS_MAC) {
    return `${mods.includes("alt") ? "⌥" : ""}${mods.includes("shift") ? "⇧" : ""}${mods.includes("mod") ? "⌘" : ""}${keyName}`;
  }
  return [...(mods.includes("mod") ? ["Ctrl"] : []), ...(mods.includes("shift") ? ["Shift"] : []), ...(mods.includes("alt") ? ["Alt"] : []), keyName].join("+");
}

/** The shortcut a key press makes, in the table's terms. */
export function shortcutOf(e: KeyboardEvent): string {
  // Letters and digits by position, since Option changes e.key on the Mac.
  const code = /^Key([A-Z])$/.exec(e.code)?.[1] ?? /^Digit(\d)$/.exec(e.code)?.[1];
  const byCode: Record<string, string> = { Equal: "=", Minus: "-", Slash: "/" };
  let key = (code ?? byCode[e.code] ?? e.key).toLowerCase();
  if (key === "+") key = "=";
  const mod = IS_MAC ? e.metaKey : e.ctrlKey;
  return [...(mod ? ["mod"] : []), ...(e.shiftKey && key !== "=" ? ["shift"] : []), ...(e.altKey ? ["alt"] : []), key].join("+");
}

/** A key press meant for a text field or an open dialog, not the app's commands. */
export function forSomethingElse(e: KeyboardEvent) {
  const t = e.target as HTMLElement | null;
  return !!t && (t.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(t.tagName) || !!document.querySelector(".dialog-overlay.open"));
}

/** Every command in the table, for shortcuts and the shortcuts list. */
export const COMMANDS: CommandDef[] = MENUS.flatMap((m) => m.items.filter((i): i is CommandDef => typeof i === "object" && "id" in i));

interface Registry {
  /** Every provider's handlers, merged; later providers win. */
  handlers: () => Handlers;
  set: (owner: string, handlers: Handlers) => void;
  run: (id: string) => boolean;
}

const RegistryContext = createContext<Registry | null>(null);
/** Changes whenever what the menus show changes, for the menus alone. */
const VersionContext = createContext(0);

/** Holds the handlers and runs the shortcuts. One, around the app. */
export function CommandProvider({ children }: { children: ReactNode }) {
  const owners = useRef(new Map<string, Handlers>());
  const shown = useRef("");
  const [version, setVersion] = useState(0);

  const registry = useMemo<Registry>(() => {
    const handlers = () => Object.assign({}, ...owners.current.values()) as Handlers;
    return {
      handlers,
      set(owner, next) {
        owners.current.set(owner, next);
        // Re-render the menus only when what they show changed.
        const all = handlers();
        const sig = Object.keys(all)
          .sort()
          .map((id) => `${id}:${all[id].enabled !== false}:${!!all[id].checked}:${all[id].label ?? ""}:${!!all[id].separatorBefore}`)
          .join("|");
        if (sig !== shown.current) {
          shown.current = sig;
          setVersion((v) => v + 1);
        }
      },
      run(id) {
        const h = handlers()[id];
        if (!h || h.enabled === false) return false;
        h.run();
        return true;
      },
    };
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (forSomethingElse(e)) return;
      const pressed = shortcutOf(e);
      const command = COMMANDS.find((c) => !c.canvasKey && (c.shortcut === pressed || c.alsoKeys?.includes(pressed)));
      // A command that can't run now leaves the key alone (⌘C with no
      // selection still copies the page's text).
      if (command && registry.run(command.id)) e.preventDefault();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [registry]);

  return createElement(RegistryContext.Provider, { value: registry }, createElement(VersionContext.Provider, { value: version }, children));
}

/** The registry, re-rendering when what the menus show changes. For menus. */
export function useMenuState(): Registry {
  useContext(VersionContext);
  return useRegistry();
}

export function useRegistry(): Registry {
  const r = useContext(RegistryContext);
  if (!r) throw new Error("useRegistry outside CommandProvider");
  return r;
}

/**
 * Gives the registry this component's handlers, fresh on every render so
 * they always see current state. `owner` names the component; its
 * handlers go when it unmounts.
 */
export function useCommands(owner: string, handlers: Handlers) {
  const registry = useRegistry();
  useLayoutEffect(() => registry.set(owner, handlers));
  useEffect(() => () => registry.set(owner, {}), [registry, owner]);
}
