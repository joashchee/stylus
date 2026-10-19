# Platform parity

Every macOS-specific mechanism Stylus uses, and what Windows and Linux
would need. Windows and Linux builds come after macOS covers the
milestone (build step 8 in Diskette's `docs/stylus-notes.md`), brought
forward on 2026-10-12: public betas for all three are built from 0.3.0
(`.github/workflows/`). Unresearched beyond naming the mechanism.

iPhone, iPad and Android (from 2026-10-19, `docs/roadmap.md` 4a) are
columns of their own; "unchecked" means written but not yet run on a
device.

| Feature | macOS | Windows | Linux | iOS and iPadOS | Android |
|---|---|---|---|---|---|
| Release build | Universal `.app`, Developer ID-signed and notarized, zipped (`scripts/build-release.sh`, on the maintainer's Mac) | NSIS installer and MSI from GitHub Actions (`.github/workflows/windows.yml`), unsigned: SmartScreen warns | `.deb` and `.rpm` from GitHub Actions (`linux.yml`, Ubuntu 22.04's glibc and newer), no AppImage (it bundles LGPL WebKitGTK) | Not yet: `tauri ios build` on the maintainer's Mac (signed by team `7PNT9QVBR8`, `bundle.iOS.developmentTeam`), TestFlight for betas | Not yet: `tauri android build`, an APK on the GitHub release |
| Shortcuts | ⌘, shown as ⇧⌘S (`shortcutLabel` in `src/lib/commands.ts`) | Ctrl, shown as Ctrl+Shift+S | Same as Windows | ⌘ with a hardware keyboard (iPadOS looks like a Mac to the webview) | Ctrl with a hardware keyboard |
| Install location for local release builds | `~/Applications/Stylus.app` via `ditto` (`scripts/build-release.sh`) | n/a (installer) | n/a (package) | n/a | n/a |
| Native menu bar from the command registry | App menu bar via Tauri's menu API (`src-tauri/src/native_menu.rs`, `src/lib/nativeMenu.ts`); standard Edit items while a text field or dialog has the keyboard | None: the in-window menu bar only (the webview keeps the keys) | Same as Windows | None: `native_menu.rs` is desktop only, and `nativeMenu.ts` skips the iPad (`IS_MOBILE` in `src/lib/formFactor.ts`) | Same as iOS |
| First-run warning | Before any window (`first_run.rs`) | Same | Same | Over a window holding a blank blue page (`public/first-run.html`): an iOS dialog needs a window to show on. OK opens the app (`start` in `lib.rs`). Unchecked | Same as iOS. Unchecked |
| Layout | Desktop, whatever the window's size | Desktop; View → Tablet Layout for a touch laptop | Same as Windows | Tablet or phone, from the screen (`src/lib/formFactor.ts`) | Same as iOS |
| Pen input | Pointer events (a tablet as a mouse) | Pointer events: barrel button, eraser end, hover (Windows Ink in WebView2) | WebKitGTK: pens usually arrive as a mouse. Unchecked | Pointer events: Apple Pencil's tip, hover (M2 iPads and later), coalesced moves (`src/lib/pen.ts`). Unchecked | Pointer events (S Pen and others), unchecked |
| Pen gestures | n/a | n/a | n/a | Apple Pencil double-tap and squeeze, `UIPencilInteraction` (`src-tauri/src/pencil.rs`); "Only Draw with Apple Pencil" read on iPadOS 17.5 and later. Unchecked | n/a: the S Pen's button is the barrel button |
| Opening files | Path from the dialog | Same | Same | The picker's copy, a `file://` URL (`local_path` in `lib.rs`). Unchecked | `content://` URIs: not read yet (roadmap 4a) |
| Saving files | Path from the dialog | Same | Same | Not yet (the share sheet; roadmap 4a) | Not yet (roadmap 4a) |

Planned, add a row when each lands: file associations for `.ANS`, `.XB`,
`.BIN` and the rest, and Quick Look previews of `.ANS`.
