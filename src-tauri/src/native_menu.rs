//! The macOS menu bar, built from the frontend's command registry
//! (`src/lib/commands.ts`, docs/ui-design.md "Menus and the macOS menu
//! bar"). The frontend sends the menus as they stand now (labels,
//! shortcuts, enabled and checked) whenever they change; this builds them
//! into the native menu bar between the app menu and the Window menu, and
//! sends each click back as the command's id (`native-menu` event).
//!
//! The webview and the menu both want ⌘C, ⌘V and ⌘Z, and text fields must
//! keep them. So the frontend also says where the keyboard is:
//!
//! - `art`: the registry's commands carry their ⌘ shortcuts.
//! - `text` (a text field has focus): Undo, Redo, Cut, Copy, Paste and
//!   Select All become the standard macOS items, which edit the field, and
//!   the other commands drop their shortcuts, as the in-window shortcuts
//!   do in a text field.
//! - `modal` (a dialog is open): the same, and every command is disabled.
//!
//! Either order of key handling then does the same thing: the webview's
//! keydown handler runs a command and stops the key, or the menu does.
//! Keys without ⌘ (tool letters, Delete, Esc) are never given to the menu,
//! which would take them from the art and from text fields alike.

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::menu::{AboutMetadata, CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Manager, Wry};

/// The event a click on a registry item sends, with the command's id.
pub const EVENT: &str = "native-menu";
/// Quit closes the window first, so unsaved changes are asked about.
const QUIT: &str = "app.quit";
/// Shown in the app menu, where macOS puts About.
const ABOUT: &str = "help.about";

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NativeItem {
    id: String,
    label: String,
    /// The registry's form (`mod+shift+s`).
    shortcut: Option<String>,
    enabled: bool,
    /// Present for items with a tick.
    checked: Option<bool>,
    /// A submenu's items (File → Open Recent), `None` a separator.
    #[serde(default)]
    children: Option<Vec<Option<NativeItem>>>,
}

impl NativeItem {
    /// This item and every item in its submenu.
    fn with_children(&self) -> Vec<&NativeItem> {
        let mut all = vec![self];
        for child in self.children.iter().flatten().flatten() {
            all.extend(child.with_children());
        }
        all
    }
}

/// Every item in the spec, submenus' items included.
fn all_items(spec: &MenuSpec) -> impl Iterator<Item = &NativeItem> {
    spec.menus.iter().flat_map(|m| m.items.iter().flatten()).flat_map(NativeItem::with_children)
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct NativeMenuDef {
    label: String,
    /// `None` is a separator.
    items: Vec<Option<NativeItem>>,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Focus {
    Art,
    Text,
    Modal,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct MenuSpec {
    menus: Vec<NativeMenuDef>,
    focus: Focus,
}

enum Item {
    Plain(MenuItem<Wry>),
    Check(CheckMenuItem<Wry>),
    Sub(Submenu<Wry>),
}

/// The menu bar as last built, to update in place when only enabled and
/// checked states change.
#[derive(Default)]
pub struct NativeMenu(Mutex<Option<(MenuSpec, HashMap<String, Item>)>>);

type StandardItem = fn(&AppHandle) -> tauri::Result<PredefinedMenuItem<Wry>>;

/// The registry's items that become the standard macOS text-editing items
/// while a text field or dialog has the keyboard.
fn standard(id: &str) -> Option<StandardItem> {
    Some(match id {
        "edit.undo" => |a| PredefinedMenuItem::undo(a, None),
        "edit.redo" => |a| PredefinedMenuItem::redo(a, None),
        "edit.cut" => |a| PredefinedMenuItem::cut(a, None),
        "edit.copy" => |a| PredefinedMenuItem::copy(a, None),
        "edit.paste" => |a| PredefinedMenuItem::paste(a, None),
        "edit.selectAll" => |a| PredefinedMenuItem::select_all(a, None),
        _ => return None,
    })
}

/// A registry shortcut as a menu accelerator, or `None` for a key without
/// ⌘ (the menu would take it from the art and from text fields).
fn accelerator(shortcut: &str) -> Option<String> {
    let parts: Vec<&str> = shortcut.split('+').collect();
    let (key, mods) = parts.split_last()?;
    if !mods.contains(&"mod") {
        return None;
    }
    let key = match *key {
        "delete" => "Backspace".to_string(),
        "escape" => "Escape".to_string(),
        k => k.to_uppercase(),
    };
    let mut out: Vec<String> = Vec::new();
    for m in mods {
        out.push(
            match *m {
                "mod" => "CmdOrCtrl",
                "shift" => "Shift",
                "alt" => "Alt",
                _ => return None,
            }
            .to_string(),
        );
    }
    out.push(key);
    Some(out.join("+"))
}

/// What the spec says about an item once the focus is taken into account.
fn shown(item: &NativeItem, focus: Focus) -> (bool, Option<String>) {
    let enabled = item.enabled && focus != Focus::Modal;
    let accel = if focus == Focus::Art { item.shortcut.as_deref().and_then(accelerator) } else { None };
    (enabled, accel)
}

/// Whether two specs build the same menus, differing only in what can be
/// updated in place (enabled, checked).
fn same_shape(a: &MenuSpec, b: &MenuSpec) -> bool {
    // An item's id, label, accelerator, whether it has a tick, and its
    // submenu's shape.
    fn item_shape(i: &Option<NativeItem>, focus: Focus) -> String {
        match i {
            None => "-".to_string(),
            Some(i) => {
                let children: Option<Vec<String>> = i.children.as_ref().map(|c| c.iter().map(|c| item_shape(c, focus)).collect());
                format!("{:?}", (&i.id, &i.label, shown(i, focus).1, i.checked.is_some(), children))
            }
        }
    }
    let shape = |s: &MenuSpec| -> Vec<(String, Vec<String>)> {
        s.menus.iter().map(|m| (m.label.clone(), m.items.iter().map(|i| item_shape(i, s.focus)).collect())).collect()
    };
    a.focus == b.focus && shape(a) == shape(b)
}

/// Builds one item (a submenu with its items), noting it in `items` to
/// update in place later.
fn make(app: &AppHandle, focus: Focus, item: &NativeItem, items: &mut HashMap<String, Item>) -> tauri::Result<Box<dyn IsMenuItem<Wry>>> {
    if focus != Focus::Art {
        if let Some(standard) = standard(&item.id) {
            return Ok(Box::new(standard(app)?));
        }
    }
    let (enabled, accel) = shown(item, focus);
    if let Some(children) = &item.children {
        let mut built: Vec<Box<dyn IsMenuItem<Wry>>> = Vec::new();
        for child in children {
            built.push(match child {
                None => Box::new(PredefinedMenuItem::separator(app)?),
                Some(c) => make(app, focus, c, items)?,
            });
        }
        let refs: Vec<&dyn IsMenuItem<Wry>> = built.iter().map(|b| b.as_ref()).collect();
        let sub = Submenu::with_id_and_items(app, &item.id, &item.label, enabled, &refs)?;
        items.insert(item.id.clone(), Item::Sub(sub.clone()));
        return Ok(Box::new(sub));
    }
    Ok(match item.checked {
        Some(checked) => {
            let i = CheckMenuItem::with_id(app, &item.id, &item.label, enabled, checked, accel.as_deref())?;
            items.insert(item.id.clone(), Item::Check(i.clone()));
            Box::new(i)
        }
        None => {
            let i = MenuItem::with_id(app, &item.id, &item.label, enabled, accel.as_deref())?;
            items.insert(item.id.clone(), Item::Plain(i.clone()));
            Box::new(i)
        }
    })
}

fn build(app: &AppHandle, spec: &MenuSpec) -> tauri::Result<(Menu<Wry>, HashMap<String, Item>)> {
    let mut items = HashMap::new();

    let pkg = app.package_info();
    let about_label = format!("About {}", pkg.name);
    let about = all_items(spec).find(|i| i.id == ABOUT);
    let about: Box<dyn IsMenuItem<Wry>> = match about {
        Some(i) => make(app, spec.focus, &NativeItem { label: about_label, ..i.clone() }, &mut items)?,
        // Before the frontend offers its About, the standard panel.
        None => Box::new(PredefinedMenuItem::about(
            app,
            Some(&about_label),
            Some(AboutMetadata { name: Some(pkg.name.clone()), version: Some(pkg.version.to_string()), ..Default::default() }),
        )?),
    };
    let quit = MenuItem::with_id(app, QUIT, format!("Quit {}", pkg.name), true, Some("CmdOrCtrl+Q"))?;
    let app_menu = Submenu::with_items(
        app,
        &pkg.name,
        true,
        &[
            about.as_ref(),
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::services(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, None)?,
            &PredefinedMenuItem::hide_others(app, None)?,
            &PredefinedMenuItem::show_all(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    let menu = Menu::with_items(app, &[&app_menu])?;
    for def in &spec.menus {
        // About moves to the app menu; tidy the separators it leaves.
        let mut entries: Vec<Option<&NativeItem>> = Vec::new();
        for entry in def.items.iter().map(Option::as_ref) {
            if entry.is_some_and(|i| i.id == ABOUT) || (entry.is_none() && entries.last().is_none_or(Option::is_none)) {
                continue;
            }
            entries.push(entry);
        }
        while entries.last().is_some_and(Option::is_none) {
            entries.pop();
        }
        let mut built: Vec<Box<dyn IsMenuItem<Wry>>> = Vec::new();
        for entry in entries {
            built.push(match entry {
                None => Box::new(PredefinedMenuItem::separator(app)?),
                Some(i) => make(app, spec.focus, i, &mut items)?,
            });
        }
        let refs: Vec<&dyn IsMenuItem<Wry>> = built.iter().map(|b| b.as_ref()).collect();
        let submenu = Submenu::with_items(app, &def.label, true, &refs)?;
        #[cfg(target_os = "macos")]
        if def.label == "Help" {
            submenu.set_as_help_menu_for_nsapp()?;
        }
        menu.append(&submenu)?;
    }

    let window = Submenu::with_items(
        app,
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(app, None)?,
            &PredefinedMenuItem::maximize(app, None)?,
            &PredefinedMenuItem::fullscreen(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, None)?,
        ],
    )?;
    #[cfg(target_os = "macos")]
    window.set_as_windows_menu_for_nsapp()?;
    // Before Help, where macOS expects it.
    let help_at = spec.menus.iter().position(|m| m.label == "Help").unwrap_or(spec.menus.len()) + 1;
    menu.insert(&window, help_at)?;
    Ok((menu, items))
}

/// Builds or updates the menu bar from the registry (macOS only; Windows
/// and Linux have the in-window menu bar alone).
#[tauri::command]
pub async fn set_native_menu(app: AppHandle, state: tauri::State<'_, NativeMenu>, spec: MenuSpec) -> Result<(), String> {
    if !cfg!(target_os = "macos") {
        return Ok(());
    }
    let mut current = state.0.lock().map_err(|e| e.to_string())?;
    if let Some((last, items)) = current.as_mut() {
        if *last == spec {
            return Ok(());
        }
        if same_shape(last, &spec) {
            let old: HashMap<&str, &NativeItem> = all_items(last).map(|i| (i.id.as_str(), i)).collect();
            for item in all_items(&spec) {
                let (Some(was), Some(built)) = (old.get(item.id.as_str()), items.get(&item.id)) else { continue };
                let (enabled, _) = shown(item, spec.focus);
                let changed_enabled = shown(was, last.focus).0 != enabled;
                let r = match built {
                    Item::Plain(i) => if changed_enabled { i.set_enabled(enabled) } else { Ok(()) },
                    Item::Sub(i) => if changed_enabled { i.set_enabled(enabled) } else { Ok(()) },
                    Item::Check(i) => (if changed_enabled { i.set_enabled(enabled) } else { Ok(()) })
                        .and_then(|_| if was.checked != item.checked { i.set_checked(item.checked.unwrap_or(false)) } else { Ok(()) }),
                };
                r.map_err(|e| e.to_string())?;
            }
            *last = spec;
            return Ok(());
        }
    }
    let (menu, items) = build(&app, &spec).map_err(|e| e.to_string())?;
    app.set_menu(menu).map_err(|e| e.to_string())?;
    *current = Some((spec, items));
    Ok(())
}

/// Sends a registry item's click to the frontend; Quit closes the window,
/// which asks about unsaved changes, and the app ends with its last window.
pub fn on_menu_event(app: &AppHandle, id: &str) {
    if id == QUIT {
        match app.get_webview_window("main") {
            Some(w) => {
                let _ = w.close();
            }
            None => app.exit(0),
        }
    } else {
        // No lock here: this runs on the main thread, which
        // `set_native_menu` may be waiting on while it holds the lock. The
        // registry ignores ids it doesn't know.
        let _ = app.emit(EVENT, id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_command_shortcuts_become_accelerators() {
        assert_eq!(accelerator("mod+shift+s").as_deref(), Some("CmdOrCtrl+Shift+S"));
        assert_eq!(accelerator("mod+alt+v").as_deref(), Some("CmdOrCtrl+Alt+V"));
        assert_eq!(accelerator("mod+=").as_deref(), Some("CmdOrCtrl+="));
        assert_eq!(accelerator("mod+/").as_deref(), Some("CmdOrCtrl+/"));
        // Keys without ⌘ stay with the art and text fields.
        assert_eq!(accelerator("p"), None);
        assert_eq!(accelerator("delete"), None);
        assert_eq!(accelerator("escape"), None);
    }

    fn item(id: &str, shortcut: Option<&str>, enabled: bool, checked: Option<bool>) -> NativeItem {
        NativeItem { id: id.into(), label: id.into(), shortcut: shortcut.map(Into::into), enabled, checked, children: None }
    }

    #[test]
    fn text_fields_and_dialogs_keep_their_keys() {
        let save = item("file.save", Some("mod+s"), true, None);
        assert_eq!(shown(&save, Focus::Art), (true, Some("CmdOrCtrl+S".into())));
        assert_eq!(shown(&save, Focus::Text), (true, None));
        assert_eq!(shown(&save, Focus::Modal), (false, None));
        assert!(standard("edit.copy").is_some() && standard("file.save").is_none());
    }

    #[test]
    fn enabled_and_ticks_update_in_place_and_focus_rebuilds() {
        let spec = |enabled, checked, focus| MenuSpec {
            menus: vec![NativeMenuDef { label: "Edit".into(), items: vec![Some(item("edit.copy", Some("mod+c"), enabled, None)), None, Some(item("tool.pencil", Some("p"), true, Some(checked)))] }],
            focus,
        };
        assert!(same_shape(&spec(true, true, Focus::Art), &spec(false, false, Focus::Art)));
        assert!(!same_shape(&spec(true, true, Focus::Art), &spec(true, true, Focus::Text)));
        let mut unticked = spec(true, true, Focus::Art);
        unticked.menus[0].items[2].as_mut().unwrap().checked = None;
        assert!(!same_shape(&spec(true, true, Focus::Art), &unticked));
    }

    #[test]
    fn a_submenu_rebuilds_when_its_items_change() {
        let recent = |files: &[&str]| MenuSpec {
            menus: vec![NativeMenuDef {
                label: "File".into(),
                items: vec![Some(NativeItem {
                    children: Some(files.iter().map(|f| Some(item(&format!("file.recent.{f}"), None, true, None))).collect()),
                    ..item("file.recent", None, !files.is_empty(), None)
                })],
            }],
            focus: Focus::Art,
        };
        assert!(same_shape(&recent(&["a"]), &recent(&["a"])));
        assert!(!same_shape(&recent(&["a"]), &recent(&["b", "a"])));
        assert_eq!(all_items(&recent(&["a", "b"])).map(|i| i.id.as_str()).collect::<Vec<_>>(), ["file.recent", "file.recent.a", "file.recent.b"]);
    }
}
