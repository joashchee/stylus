/**
 * The macOS menu bar, from the command registry (docs/ui-design.md, "Menus
 * and the macOS menu bar"; src-tauri/src/native_menu.rs builds it). The
 * in-window menu bar stays: it's part of the look, and all the web app and
 * Windows and Linux have.
 *
 * Sends the menus whenever what they show changes, and where the keyboard
 * is, so a text field or dialog keeps ⌘C, ⌘V and ⌘Z (native_menu.rs has
 * the rules). Runs a command chosen in the menu bar through the registry.
 */
import { useEffect, useState } from "react";
import { onNativeMenu, setNativeMenu, type NativeItem, type NativeMenuSpec } from "./backend";
import { IS_MAC, itemEnabled, menuItems, MENUS, useMenuState, type MenuItem } from "./commands";

type Focus = NativeMenuSpec["focus"];

function dialogOpen() {
  return !!document.querySelector(".dialog-overlay.open");
}

function focusNow(): Focus {
  if (dialogOpen()) return "modal";
  const el = document.activeElement as HTMLElement | null;
  return el && (el.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(el.tagName)) ? "text" : "art";
}

/** Where the keyboard is, kept current as focus moves and dialogs open. */
function useFocus(): Focus {
  const [focus, setFocus] = useState<Focus>(focusNow);
  useEffect(() => {
    const update = () => setFocus(focusNow());
    // After the focus has gone wherever it's going.
    const later = () => window.setTimeout(update, 0);
    document.addEventListener("focusin", update);
    document.addEventListener("focusout", later);
    // A dialog opening or closing changes a class, not always the focus.
    const observer = new MutationObserver(update);
    observer.observe(document.body, { subtree: true, attributes: true, attributeFilter: ["class"] });
    return () => {
      document.removeEventListener("focusin", update);
      document.removeEventListener("focusout", later);
      observer.disconnect();
    };
  }, []);
  return focus;
}

const inTauri = "__TAURI_INTERNALS__" in window;

function nativeItem(item: MenuItem | "-"): NativeItem | null {
  if (item === "-") return null;
  return {
    id: item.id,
    label: item.label,
    shortcut: item.shortcut,
    enabled: itemEnabled(item),
    checked: item.handler?.checked,
    children: item.children?.map(nativeItem),
  };
}

/** Keeps the macOS menu bar in step with the registry. Once, under `CommandProvider`. */
export function useNativeMenu() {
  const registry = useMenuState();
  const focus = useFocus();
  const handlers = registry.handlers();
  const spec: NativeMenuSpec = {
    focus,
    menus: MENUS.map((menu) => ({
      label: menu.label,
      items: menuItems(menu, handlers).map(nativeItem),
    })),
  };
  const key = JSON.stringify(spec);

  useEffect(() => {
    if (!IS_MAC || !inTauri) return;
    setNativeMenu(JSON.parse(key) as NativeMenuSpec).catch((e) => console.error("native menu:", e));
  }, [key]);

  useEffect(() => {
    if (!IS_MAC || !inTauri) return;
    const unlisten = onNativeMenu((id) => {
      if (!dialogOpen()) registry.run(id);
    });
    return () => void unlisten.then((f) => f());
  }, [registry]);
}
