# Stylus

ANSI art and animation tool, and the **ANSIapps theme's workshop**:
ansiapps' second hero app after Diskette (`~/Coding/diskette`). It opens
every legacy ASCII/ANSI art format, makes art five ways (by hand, image →
ANSI, ASCII → ANSI, text → ANSI, seeded random), plays and makes
ANSImations, and is where the theme pack every ansiapps app uses is
drawn, contrast-checked and exported. Tauri 2 + React/TypeScript + Rust
on icy_tools, macOS first. The design record lives on Diskette's side in
**`~/Coding/diskette/docs/stylus-notes.md`**. Read it before any
architectural change. This file is the condensed operating rules only.

**`CHANGELOG.md`**: add a bullet whenever a change lands.

## Current milestone: Phase 1, open and draw (started 2026-09-30)

The rollout is **`docs/roadmap.md`** (finalised 2026-09-30): 1 open and
draw (first public beta), 2 theme workshop, 3 make and animate (1.0), 4
everywhere and beyond, with a "Not planned" list of survey features
Stylus won't build. The screen each phase builds is
**`docs/ui-design.md`**: designed in 8×16 cells on the ANSIapps theme
first, Art/Make/Theme/Lab workspaces, one Problems list, one command
registry. The `ansi-art-app-roadmap-*.md` and
`ansi-editor-ui-design-chatgpt.md` surveys are reference only. Tick
roadmap items as they land, with the date.

**Where Phase 1 is** (details in the roadmap): the viewer's open items
are done except the App Testing pass on real art. The editor has its
editing core with undo by stroke, New, Save and Save As in all seven
formats with the loss warning and SAUCE fields, the corpus round trip,
and every Phase 1 tool (Pencil, Eraser, Line, Rectangle, Box,
Half-block, Type, Select, Fill, Pick) with selection (cut, copy, paste,
move, flip, fill, clear), and the menu bar over one command registry, mirrored in the macOS menu
bar (`src-tauri/src/native_menu.rs`, `src/lib/nativeMenu.ts`), and
recent files and crash-recovery autosave (`src-tauri/src/files.rs`,
`stylus-core/src/recovery.rs`), and PNG export
(`stylus-core/src/export.rs`). Next: the contrast lint, and measuring
the IPC hot path on the M1.

**Viewer history, still true:** checked against libansilove on Sixteen
Colors packs (`scripts/compare-ansilove.sh`, corpus in the scratch
folder, never the repo); every difference is one listed in the script's
header. icy_tools#187 (merged as `be236b5`): DEL prints ⌂, iCE leaves
24-bit backgrounds alone, **BEL stays a control code** by the
maintainer's choice, so files with BEL differ from ansilove. The
"77 rows" file (`c-mfs - blender.ans`) was an EGA 8×14 font that ansilove
draws at 16 px; Stylus is right.

## Non-negotiable rules

1. **No server, no telemetry, no accounts.** No art or image leaves the
   machine. The web app processes everything in the browser.
2. **Open source and free, always. MIT** for the app and `theme/`.
   **No GPL/AGPL/LGPL dependencies**, so `stylus-core` stays usable by the
   closed-source apps (so no chafa). Run `scripts/license-scan.py`
   whenever `Cargo.lock` or `package-lock.json` changes.
3. **Bundle ID `com.ansiapps.stylus`.** Never change it once a build
   ships: it keys the app-data folder.
4. **Opening never changes the file.** Saving writes a new file, or
   replaces the old one only when the user asks. A save never appends a
   second SAUCE record.
5. **Seeded generation is reproducible forever**: our own PRNG
   (SplitMix64 or PCG32, never `rand`'s default), integer math in
   generators, and generator name, version, seed and settings in SAUCE
   comments.
6. **No generative AI.** All five ways of making art are deterministic
   algorithms.
7. **The art canvas shows the art's own colors.** The theme's 16-color
   rule and the contrast list cover the app's own screens only. Every UI
   text/background pair comes from `docs/ansiapps-color-contrast.md`, in
   both themes.
8. **icy_tools is pinned** to one git revision (`stylus-core/Cargo.toml`,
   and `ICY_TOOLS_REV` in `stylus-core/src/lib.rs`, kept in step).
   Upgrade deliberately, rerun the license scan, and send fixes upstream
   rather than carrying a fork.
9. **Never commit or ship third-party art.** The test corpus (Sixteen
   Colors) is cloned outside the repo. Commit only test files Stylus
   made.
10. **macOS first.** Add a `docs/platform-parity.md` row for every
    macOS-specific mechanism.

## Where things live

- `stylus-core/`: the Tauri-free Rust crate on icy_engine. All art logic
  goes here, never in `src-tauri`, so the web build shares it.
  - `src/document.rs`: `Document`, a file opened for viewing and
    editing: its info, the render settings (9-px, iCE, aspect),
    `render_rows` (a band of rows through icy_engine's renderer) and
    `render_cells` (just the cells an edit changed). UTF-8 text is shown
    as CP437 codes, since the IBM fonts draw those. Render modes
    never change the data, except that iCE moves the attribute's high bit
    between blink and bright background, which is reversible. Parsed
    formats end at their last row with content; binary formats keep their
    height.
  - `src/edit.rs`: editing. Stylus's own layer over icy_engine's buffer
    (not `icy_engine_edit`: GUI, tokio and websocket dependencies). Cell
    edits in strokes (one undo step per pencil drag or typed character),
    resize, `save` through icy_engine's writers (`SAVE_FORMATS`, cell-exact,
    one fresh SAUCE record with the file's date) and `save_losses`, the
    plain-language list of what a format would lose. `TEXT_FONTS`: the
    Amiga fonts for text files without SAUCE.
  - `src/tools.rs`: the drawing tools and selection: shapes (`Shape`,
    drawn with `draw_shape`, which retracts and redraws the same stroke
    at each move of a drag), flood fill, the half-block brush, and
    `Clip` / `SelectionOp` (copy, paste, move, flip with mirrored
    characters, fill, clear). Every edit goes through `Document::commit`
    in `edit.rs`.
  - `src/export.rs`: PNG export, `PngExport` fed a band of rows at a
    time (8/9-px, aspect stretch by integer area weights).
  - `src/recovery.rs`: the autosave snapshot, lossless (IcyDraw plus the
    file's own SAUCE record, format name and render settings), reopened
    as unsaved changes.
  - `src/sauce.rs`: `SauceInfo`, every SAUCE field, decoded from CP437.
  - `src/convert/`: Image to ANSI. One module per ported open-source
    converter (each with its `ConverterInfo`: origin, license, commit,
    settings, adaptations) and its license text in `licenses/`; `grid.rs`
    is the converters' `.ANS` writer (no line breaks: rows fill the SAUCE
    width; the editor saves through icy_engine);
    `util.rs`, `nfnt.rs`, `magick.rs` (ImageMagick) and `stbir.rs`
    (stb_image_resize2) hold the ported resamplers. Keep ports faithful
    to the original's arithmetic, and check a new one cell by cell
    against the original (`examples/convert_image.rs` dumps cells), built
    for x86-64 or with `-ffp-contract=off` (arm64 builds fuse
    multiply-adds).
    `docs/image-to-ansi-converters.md` is the survey.
  - `examples/render_png.rs`, `examples/png_diff.rs`: for
    `scripts/compare-ansilove.sh`. `examples/roundtrip.rs`: for
    `scripts/roundtrip-corpus.sh` (open, save, reopen, compare pixels).
  - Later: converters, generators, the animation encoder, the contrast
    checker, the theme-pack model and `stylus-render`.
- `src-tauri/src/lib.rs`: Tauri commands wrapping the core, for file I/O
  and OS integration only. `files.rs`: the app-data folder's
  `recent.json` and `Recovery/` autosaves (never the user's file).
- `src-tauri/src/first_run.rs`: the in-development warning on first run
  (`docs/ansiapps-theme.md`). The main window is `"create": false` and
  built in `setup` only once it's accepted.
- `src/lib/backend.ts`: the one frontend adapter over the core (Tauri now;
  WASM for the web and the editor's hot path later).
- `src/components/ArtViewer.tsx`: draws the art, one `<canvas>` per band
  of about 2048 px (one canvas can't hold a long ANSI), with a
  determinate progress bar over the rows. Aspect and zoom are CSS
  scaling. Blink alternates two frames once a second, never under reduced
  motion. For editing it also redraws a rectangle of cells and reports
  the cell under the pointer.
- `src/components/ArtEditor.tsx`: the Art workspace (Phase 1):
  tools and selection, F-key strip (`lib/cp437.ts` has the sets and the CP437 table for
  labels), Character/Colors/SAUCE panel, status bar. Edits are queued so
  they reach the core in order. `SaveDialog.tsx`: Save and Save As with
  the loss list and SAUCE fields. `SaucePanel.tsx` shows the record.
- `src/lib/commands.ts`: **the command registry**. `MENUS` names every
  command, its shortcut and menu place; components hand in what can run
  now with `useCommands` (run, enabled, checked); `CommandProvider` runs
  the shortcuts (not in text fields or dialogs, and a disabled command
  leaves the key alone). Add a command here, never a separate keydown
  listener. `components/MenuBar.tsx` draws it (and `ShortcutList` for
  Help), and `lib/nativeMenu.ts` sends it to the macOS menu bar; canvas-only keys (tool letters, Delete, Esc) are marked
  `canvasKey` and handled by the canvas.
- `src/components/ImageToAnsi.tsx`: the Image to ANSI tab (results drawn
  by `ArtThumb.tsx`). Dropping an image on the window opens it here;
  dropping art opens it in the Art workspace.
- `src/App.tsx`: the screen: workspaces, the open document (with its
  path, and whether Save may replace it), New, Save, the discard-changes
  prompt, closing with unsaved changes. `STARTUP_STEPS` feeds
  `StartupScreen`; add a step for each launch-time load (font atlases,
  recent files, the theme pack).
- `theme/`: the ANSIapps theme pack (MIT). Empty until step 4.
- **Two themes, default flipped**: Stylus opens in **ANSIapps**, with
  modern as the option in the gear menu. Key `stylus.theme`; with nothing
  stored, `index.html` (the `<html data-theme>` attribute and the
  no-flash script) and `lib/theme.ts` assume ANSIapps. Follow
  `docs/ansiapps-theme.md`. Stylus's accent is fuchsia `#e879f9`,
  light magenta in the ANSIapps theme (provisional).
- **The theme's font is IBM VGA 8x16 by VileR**, CC BY-SA 4.0, in
  `public/fonts/ansiapps/` with its `LICENSE.TXT` and `README.txt`,
  credited in About. Ship it unmodified as its own file: never subset,
  convert, rename or inline it. Theme mode will rasterize it at run time.
- Shared with Diskette and kept identical: `ProgressBar.tsx`,
  `ActivityStatus.tsx`, `lib/activity.ts`, `lib/estimate.ts`,
  `StartupScreen.tsx` and the `index.html` splash, `Dialog.tsx`, the gear
  menu, `AppTesting.tsx`, the tokens in `index.css`, and
  `docs/ansiapps-theme.md` / `docs/ansiapps-color-contrast.md` (Diskette's
  copies are the source of truth).
- **Feedback for every action** through `runActivity` + `ProgressBar`.
  Countable work gets a determinate bar (PNG export rows, conversion
  cells, animation frames, theme-pack parts).
- Version: `package.json` is the source of truth. A bump also touches
  `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and `Cargo.lock`
  (and `stylus-core/Cargo.toml` when the core changes).

## Open before step 2

- **WASM check (the gate): passes since icy_tools `2e4e2b1`, checked
  2026-09-30, rechecked at the current pin `da0d287`** (`cargo check --target wasm32-unknown-unknown` in
  `stylus-core/`, default features). Our icy_tools#186 proposed `net` and
  `archives` feature flags; the maintainer closed it and removed the
  blockers upstream instead, preferring Rust-only fixes over flags:
  `TerminalEmulation` moved to a small `icy_terminal_emulation` crate
  (icy_board), `.icy` compression uses pure-Rust `ruzstd`, and unarc-rs
  0.7 and retrofont need no C code. No icy_net, tokio, rustls, zstd-sys or
  unrar left in the tree, and no wasm-capable clang needed. Use icy_engine
  with its default features; don't reintroduce feature flags.
  - **Still open for the web:** rayon compiles for wasm32 but can't spawn
    threads there without extra setup, so a parallel path hit from the
    browser would fail at run time. The maintainer offered to look at it
    if Stylus runs into it.
- **Art-file rendering (icy_tools#187, merged as `be236b5`):** see the
  milestone above. Follow icy_engine's choice on BEL rather than patching
  around it.
- **icy crates without a license field**: `icy_engine` (repo MIT OR
  Apache-2.0), and `codepages`, `icy_terminal_emulation` (icy_board repo,
  Apache-2.0).
  Apache-2.0 needs its notice kept. Worth asking upstream to add the
  fields.
- The other open questions (icy_engine font origins, `icy_engine_edit`
  vs our own editing model, bundled TDF/FIGlet licenses, the accent) are
  in the notes.

## Testing

Run all of these before calling a change done:
`cargo test --manifest-path stylus-core/Cargo.toml`,
`cargo test --manifest-path src-tauri/Cargo.toml`, `npx tsc --noEmit`,
and `npm run build`. Add App Testing items (`src/lib/testChecklist.ts`)
with every feature, and check new UI in both themes.

## Build notes

- Releases only through `scripts/build-release.sh` (strips `$HOME` paths
  from the binary, same as Diskette). "Build the macOS app" means the
  `.app` only (`scripts/build-release.sh --bundles app`); a DMG only on
  request. Every release build is installed to `~/Applications/Stylus.app`
  with `ditto`. Claude's sandbox can't write there, so when the script
  reports it couldn't install, tell the user to run the `ditto` line.
- A release must be Developer ID-signed and notarized (itch.io, and the
  web app at `stylus.ansiapps.com`).
- **In Claude's sandbox**, `~/.cargo` and `~/.rustup` aren't writable:
  cargo can't fetch git dependencies (icy_tools) into the normal cache, and
  `rustup target add` fails. Run cargo with `CARGO_HOME` pointed at the
  scratchpad, and ask the user to run `rustup` themselves.
