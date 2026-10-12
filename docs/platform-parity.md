# Platform parity

Every macOS-specific mechanism Stylus uses, and what Windows and Linux
would need. Windows and Linux builds come after macOS covers the
milestone (build step 8 in Diskette's `docs/stylus-notes.md`), brought
forward on 2026-10-12: public betas for all three are built from 0.3.0
(`.github/workflows/`). Unresearched beyond naming the mechanism.

| Feature | macOS | Windows | Linux |
|---|---|---|---|
| Release build | Universal `.app`, Developer ID-signed and notarized, zipped (`scripts/build-release.sh`, on the maintainer's Mac) | NSIS installer and MSI from GitHub Actions (`.github/workflows/windows.yml`), unsigned: SmartScreen warns | `.deb` and `.rpm` from GitHub Actions (`linux.yml`, Ubuntu 22.04's glibc and newer), no AppImage (it bundles LGPL WebKitGTK) |
| Shortcuts | ⌘, shown as ⇧⌘S (`shortcutLabel` in `src/lib/commands.ts`) | Ctrl, shown as Ctrl+Shift+S | Same as Windows |
| Install location for local release builds | `~/Applications/Stylus.app` via `ditto` (`scripts/build-release.sh`) | n/a (installer) | n/a (package) |
| Native menu bar from the command registry | App menu bar via Tauri's menu API (`src-tauri/src/native_menu.rs`, `src/lib/nativeMenu.ts`); standard Edit items while a text field or dialog has the keyboard | None: the in-window menu bar only (the webview keeps the keys) | Same as Windows |

Planned, add a row when each lands: file associations for `.ANS`, `.XB`,
`.BIN` and the rest, and Quick Look previews of `.ANS`.
