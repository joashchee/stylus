/**
 * The in-window menu bar (docs/ui-design.md, "Menus and the macOS menu
 * bar"), drawn from the command registry (lib/commands.ts). Turbo Vision
 * style in the ANSIapps theme: a light-gray bar, access letters in blue,
 * the open menu and the item under the pointer in green.
 *
 * Mouse: click a title to open its menu; while one is open, pointing at
 * another title opens that one. Keyboard: Option/Alt + an access letter
 * opens a menu, arrows move through items and menus, Enter runs, Esc
 * closes.
 *
 * On macOS it also keeps the native menu bar in step (lib/nativeMenu.ts).
 */
import { Fragment, useEffect, useRef, useState, type KeyboardEvent as ReactKeyboardEvent, type ReactNode } from "react";
import { forSomethingElse, IS_MAC, menuItems, MENUS, shortcutLabel, useMenuState, type CommandDef, type MenuId, type MenuItem } from "../lib/commands";
import { useNativeMenu } from "../lib/nativeMenu";

function enabled(item: MenuItem | "-"): item is MenuItem {
  return item !== "-" && !!item.handler && item.handler.enabled !== false;
}

/** The title with its access letter marked. */
function AccessLabel({ label, access }: { label: string; access: string }) {
  const at = label.toLowerCase().indexOf(access);
  if (at < 0) return <>{label}</>;
  return (
    <>
      {label.slice(0, at)}
      <span className="access-letter">{label[at]}</span>
      {label.slice(at + 1)}
    </>
  );
}

export function MenuBar({ start, end }: { start?: ReactNode; end?: ReactNode }) {
  const registry = useMenuState();
  const handlers = registry.handlers();
  useNativeMenu();
  const [open, setOpen] = useState<MenuId | null>(null);
  const barRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLButtonElement | null)[]>([]);
  const titleRefs = useRef<Partial<Record<MenuId, HTMLButtonElement | null>>>({});
  /** Focus the first item once a menu opens from the keyboard. */
  const focusFirst = useRef(false);
  /** What had the keyboard before the menus took it (usually the canvas). */
  const returnFocus = useRef<HTMLElement | null>(null);

  function rememberFocus() {
    const el = document.activeElement as HTMLElement | null;
    if (el && !barRef.current?.contains(el)) returnFocus.current = el;
  }

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (barRef.current && !barRef.current.contains(e.target as Node)) setOpen(null);
    };
    document.addEventListener("mousedown", onDown);
    return () => document.removeEventListener("mousedown", onDown);
  }, [open]);

  useEffect(() => {
    if (open && focusFirst.current) {
      focusFirst.current = false;
      itemRefs.current.find((b) => b && !b.disabled)?.focus();
    }
  }, [open]);

  // Option/Alt + an access letter opens its menu.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!e.altKey || e.metaKey || e.ctrlKey || e.shiftKey || forSomethingElse(e)) return;
      const letter = /^Key([A-Z])$/.exec(e.code)?.[1]?.toLowerCase();
      const menu = MENUS.find((m) => m.access === letter);
      if (!menu) return;
      e.preventDefault();
      rememberFocus();
      focusFirst.current = true;
      setOpen(menu.id);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  function close(refocus: boolean) {
    const was = open;
    setOpen(null);
    if (refocus && was) titleRefs.current[was]?.focus();
  }

  function run(id: string) {
    setOpen(null);
    // After the menu closes, with the keyboard back where it was.
    window.setTimeout(() => {
      returnFocus.current?.focus();
      registry.run(id);
    }, 0);
  }

  function onMenuKey(e: ReactKeyboardEvent) {
    const menuIndex = MENUS.findIndex((m) => m.id === open);
    const buttons = itemRefs.current.filter((b): b is HTMLButtonElement => !!b && !b.disabled);
    const at = buttons.indexOf(e.currentTarget as HTMLButtonElement);
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const next = buttons[(at + (e.key === "ArrowDown" ? 1 : buttons.length - 1)) % buttons.length];
      next?.focus();
    } else if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
      e.preventDefault();
      const next = MENUS[(menuIndex + (e.key === "ArrowRight" ? 1 : MENUS.length - 1)) % MENUS.length];
      focusFirst.current = true;
      setOpen(next.id);
    } else if (e.key === "Escape") {
      e.preventDefault();
      close(true);
    } else if (e.key === "Tab") {
      setOpen(null);
    }
  }

  return (
    <div className="menu-bar" ref={barRef} data-testid="menu-bar" onPointerDownCapture={rememberFocus}>
      {start}
      <div className="menu-titles" role="menubar" aria-label="Menus">
        {MENUS.map((menu) => {
          const items = open === menu.id ? menuItems(menu, handlers) : [];
          if (open === menu.id) itemRefs.current = [];
          return (
            <div key={menu.id} className="menu-wrap">
              <button
                type="button"
                role="menuitem"
                aria-haspopup="menu"
                aria-expanded={open === menu.id}
                className={`menu-title${open === menu.id ? " open" : ""}`}
                data-testid={`menu-${menu.id}`}
                ref={(el) => {
                  titleRefs.current[menu.id] = el;
                }}
                onClick={() => setOpen(open === menu.id ? null : menu.id)}
                onPointerEnter={() => open && open !== menu.id && setOpen(menu.id)}
                onKeyDown={(e) => {
                  if (e.key === "ArrowDown") {
                    e.preventDefault();
                    focusFirst.current = true;
                    setOpen(menu.id);
                    if (open === menu.id) itemRefs.current.find((b) => b && !b.disabled)?.focus();
                  } else if (e.key === "Escape") {
                    setOpen(null);
                  }
                }}
              >
                <AccessLabel label={menu.label} access={menu.access} />
              </button>
              {open === menu.id && (
                <div className="menu-dropdown" role="menu" aria-label={menu.label}>
                  {items.map((item, i) =>
                    item === "-" ? (
                      <div key={`sep-${i}`} className="menu-sep" role="separator" />
                    ) : (
                      <Fragment key={item.id}>
                        <button
                          type="button"
                          role={item.handler?.checked !== undefined ? "menuitemcheckbox" : "menuitem"}
                          aria-checked={item.handler?.checked !== undefined ? item.handler.checked : undefined}
                          className="menu-item menu-command"
                          data-testid={`command-${item.id}`}
                          disabled={!enabled(item)}
                          ref={(el) => {
                            itemRefs.current.push(el);
                          }}
                          onClick={() => run(item.id)}
                          onKeyDown={onMenuKey}
                        >
                          <span className="menu-check" aria-hidden="true">
                            {item.handler?.checked ? "✓" : ""}
                          </span>
                          <span className="menu-label">{item.label}</span>
                          <span className="menu-shortcut">{item.shortcut ? shortcutLabel(item.shortcut) : ""}</span>
                        </button>
                      </Fragment>
                    ),
                  )}
                </div>
              )}
            </div>
          );
        })}
      </div>
      <div className="menu-bar-end">{end}</div>
    </div>
  );
}

/** Help → Keyboard shortcuts: every shortcut in the registry, by menu. */
export function ShortcutList() {
  return (
    <div className="shortcut-list" data-testid="shortcut-list">
      {MENUS.map((menu) => {
        const rows = menu.items.filter((i): i is CommandDef => typeof i === "object" && "id" in i && !!i.shortcut);
        if (rows.length === 0) return null;
        return (
          <section key={menu.id}>
            <h3>{menu.label}</h3>
            <dl>
              {rows.map((c) => (
                <Fragment key={c.id}>
                  <dt>{c.label}</dt>
                  <dd className="mono">
                    {shortcutLabel(c.shortcut!)}
                    {c.canvasKey ? " (on the art)" : ""}
                  </dd>
                </Fragment>
              ))}
            </dl>
          </section>
        );
      })}
      <section>
        <h3>On the art</h3>
        <dl>
          <dt>Move the cursor</dt>
          <dd className="mono">Arrows, Home, End, Page Up, Page Down</dd>
          <dt>Select from the cursor</dt>
          <dd className="mono">Shift + arrows</dd>
          <dt>Type a character from the strip</dt>
          <dd className="mono">F1–F10</dd>
          <dt>Next or previous character set</dt>
          <dd className="mono">{IS_MAC ? "⌃← ⌃→" : "Ctrl+← Ctrl+→"}</dd>
          <dt>Pick a cell's character and colors</dt>
          <dd className="mono">{IS_MAC ? "Option" : "Alt"}-click</dd>
          <dt>Leave Type for the last tool</dt>
          <dd className="mono">Esc</dd>
        </dl>
      </section>
    </div>
  );
}
