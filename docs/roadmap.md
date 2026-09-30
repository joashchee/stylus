# Stylus roadmap: the four phases

**Finalised 2026-09-30.** This is the rollout plan from here to a
finished Stylus. It consolidates two feature surveys,
`ansi-art-app-roadmap-chatgpt.md` and `ansi-art-app-roadmap-gemini.md`
(both written for a generic ANSI editor), with the design record in
Diskette's `docs/stylus-notes.md` and its eight-step build order. Where
they disagree, the notes and `CLAUDE.md`'s rules win, then this doc,
then the surveys. The surveys stay in `docs/` as reference only.

## How the surveys were used

Both surveys phase features by *history* (what every editor has, what
the big editors have, what any editor ever had, what nobody built).
That's a good checklist but not a rollout: it would put the theme
workshop, the reason Stylus exists for the family, nowhere, and it would
build network collaboration and AI, which Stylus rules out. So the
phases below are phased by **what ships and who it unblocks**, and each
survey item is either placed in a phase or listed under "Not planned"
with the reason.

Kept from the surveys:

- **ANSI is not the internal model** (ChatGPT, "Core Architecture";
  `known-ansi-editors.md` §24). The document is cells + palette + font +
  SAUCE, with layers and frames, and formats are adapters around it.
  icy_engine's buffer already works this way; Phase 1 decides whether
  its editing layer (`icy_engine_edit`) is ours too.
- **The checklist of editing basics** (canvas, cells, tools, selection,
  files, viewport) for Phase 1, and the reference-editor features
  (TheDraw fonts, half-blocks, transforms, iCE, layers) for Phases 1–3.
- **Historical playback** (baud-rate simulation, delta-encoded
  ANSImations), which the Gemini survey calls speculative but Stylus's
  notes already plan for Phase 3.
- **The research ideas** (format capability analysis, loss reports,
  round-trip testing, emulation profiles, a playback lab, fuzzing) for
  Phase 4, where Stylus goes beyond the historical editors.

The screen each phase builds is in **`docs/ui-design.md`**.

## The phases at a glance

| Phase | Name | Build steps (notes) | Ships | Unblocks |
|---|---|---|---|---|
| 1 | **Open and draw** | 2 (finish), 3 | First public macOS beta | Everything after: the document model and editor |
| 2 | **The theme workshop** | 4 | Theme pack v1; Diskette on it | Every ansiapps app's ANSI look |
| 3 | **Make and animate** | 5, 6 | **Stylus 1.0** for macOS | The full "five ways to make art" promise |
| 4 | **Everywhere, and beyond** | 7, 8, then new | Web app, Windows, Linux; the lab | Users off macOS; the preservation tools |

The step order is the notes' own; the phases group it into releases.
Phase 2 still comes before the converters and animation because the
family is waiting on the theme pack.

Every phase ends with the usual gate: the four test commands in
`CLAUDE.md`, App Testing items for each feature checked in both themes,
the license scan after any lockfile change, and a signed, notarized
build from `scripts/build-release.sh`.

---

## Phase 1: Open and draw

**Goal:** Stylus opens every file it can, and you can draw, edit and
save ANSI art by hand as well as in the classic editors' basics.
**Release:** 0.x, the first public macOS beta on itch.io (Developer
ID-signed, notarized, the in-development warning on).

### 1a. Finish the viewer (build step 2)

- [ ] App Testing pass on real art.
- [x] Explain `blndr2020/c-mfs - blender.ans` (77 rows vs 88). **Done
  2026-09-30:** Stylus has all 88 rows. The file's SAUCE font is "IBM
  EGA" (8×14), which Stylus uses (1232 px); ansilove draws 8×16
  (1408 px), and 1232 px read as 16-px rows looked like 77. An intended
  difference, now in `scripts/compare-ansilove.sh`'s header.
- [x] **Amiga ASCII:** a Font menu for `.ASC`/`.TXT` without SAUCE (the
  eight Amiga fonts icy_engine has), remembered per extension; SAUCE
  still wins. **Done 2026-09-30.** Also found and fixed: UTF-8 text
  files showed ▀ ▄ █ as blanks (read as Unicode, drawn by a CP437 font);
  they're now shown as their CP437 codes.
- [ ] Viewer basics from the surveys not yet in: fit-to-window,
  actual-size, a status bar (size in cells, font, SAUCE flags).

### 1b. The decisions that start the editor

- [x] **Editing model: our own, over icy_engine's buffer** (decided
  2026-09-30, `stylus-core/src/edit.rs`). `icy_engine_edit` depends on a
  GUI toolkit (icy_ui, on by default), tokio "full" and websockets for
  its collaboration server: none of that builds for the web, and
  networking is against rule 1. icy_engine itself supplies what the
  editor needs: every Phase 1 format's writer, SAUCE on save, and the
  format compatibility check behind the save-loss warning. Undo is
  Stylus's: one step per stroke (a pencil drag, a typed character) or
  resize.
- [ ] **Hot path benchmark on the M1:** the WASM core in the webview vs
  Tauri IPC per stroke, as the notes recommend. **Started:** the first
  editor uses IPC (an edit, then a render of just the changed cells, drawn
  by icy_engine's own renderer rather than a glyph atlas, so the canvas
  matches the viewer exactly). Every call goes through `lib/backend.ts`,
  so moving to WASM changes one file. Measure a fast pencil drag at
  400% on the M1; move to WASM if it lags.
- [ ] **The model carries layers and frames from day one,** even though
  their UI comes in Phase 3, so no later file format change is needed.

### 1c. The editor (build step 3)

- [ ] **Canvas:** new document (80×25, 80×50, 132, 160 columns, or any
  size), resize, crop, grow-as-you-draw height, scrolling, and
  keyboard navigation (arrows, Home/End, Page Up/Down, jump to line).
  **Done:** new, grow-as-you-draw (Enter or Down on the last row),
  scrolling, navigation; resize in the core (undoable). **Left:** resize
  and crop in the UI, jump to line.
- [ ] **Typing on the grid:** overwrite and insert modes, the
  **F-key character sets** (TheDraw/PabloDraw convention, editable), a
  CP437 character picker, entry by character code, and pasting text.
  **Done:** overwrite, ten F-key sets with a clickable strip, the
  picker. **Left:** insert mode, editing the sets, entry by code, paste.
- [ ] **Color:** the 16 colors with foreground/background swatches,
  pick color from a cell (eyedropper), paint color only or character
  only, swap foreground/background, **iCE on/off** per document, and
  24-bit where the format allows (PabloDraw `.ANS`, XBin palettes).
  **Done:** all but 24-bit.
- [x] **Tools:** pencil, eraser, line, rectangle (outline and filled),
  box drawing with the single/double line sets, flood fill, and the
  **half-block brush** (80×(2×rows) "pixels", Moebius). **Done
  2026-09-30** (`stylus-core/src/tools.rs`). A dragged shape is redrawn
  by the core at each move (the stroke is taken back and drawn again),
  so the preview is the art and the drag is one undo step. Fill takes
  cells joined up, down, left and right with exactly the same character
  and colors. The flyouts wait for the rail's redesign with the menu bar.
- [x] **Selection:** rectangular select, select all, cut/copy/paste,
  move, delete, fill, flip horizontal/vertical, and transparent vs
  opaque paste. **Done 2026-09-30.** The clipboard is Stylus's own, keeps
  every attribute (24-bit, blink) and pastes into any open document; a
  transparent paste skips blanks on black. Flips mirror the characters
  (▌▐, ▀▄, box corners, brackets) in the PC fonts. Copying to the
  system clipboard as text is left for the paste-text item above.
- [x] **Undo/redo**, unlimited within the session.
- [ ] **Viewport:** zoom, grid, cursor and cell coordinates in the
  status bar, current character and attribute.
- [ ] **The editor's window** (`docs/ui-design.md`): menu bar (plus the
  macOS native one), Art and Make workspaces in place of the Viewer
  tab, document tabs, tool rail, the F-key strip, the context panel
  (Character, Colors, SAUCE, Properties, Problems), status bar, and one
  command registry behind menus and shortcuts.
  **Done 2026-09-30:** the command registry (`src/lib/commands.ts`: every
  command's name, shortcut and menu in one table; components supply what
  can run now) behind the in-window menu bar (`MenuBar.tsx`, File, Edit,
  Draw, Select, Colors, View, Help, with access letters and ticks) and
  every ⌘ shortcut; Help → Keyboard Shortcuts from the same table; the
  Art and Make switcher and the gear at the menu bar's right; the
  toolbar gone into the menus; the art's name in the window title and
  the status bar. The macOS native menu bar from the same registry
  (**done 2026-09-30**, `src-tauri/src/native_menu.rs`): the frontend
  sends the menus as they stand and where the keyboard is; on the art
  the commands carry their ⌘ shortcuts, and in a text field or dialog
  Undo, Redo, Cut, Copy, Paste and Select All become the standard macOS
  items and the rest drop their shortcuts (greyed out under a dialog),
  so the keys do the same whichever of the webview and the menu sees
  them first. Keys without ⌘ never go to the menu. Quit closes the
  window, so unsaved changes are asked about. Still to try in the app
  (the App Testing "Native menu" items). **Left:** document tabs, the
  Properties and Problems tabs, the rail's icons and flyouts, and the
  resizable panel.
- [x] **Files:** new, save, save as, recent files, unsaved-change
  prompts, and autosave to the app-data folder for crash recovery
  (never over the user's file). **Done 2026-09-30:** new, save (asks the
  first time it would replace the file), save as, unsaved-change prompts
  on open, new and closing the window; a replaced file is written beside
  it and moved over it. Recent files: the last ten opened or saved, in
  `recent.json` in the app-data folder, as File → Open Recent (a
  submenu in the command registry, both menu bars) and on the empty
  screen. Autosave (`stylus-core/src/recovery.rs`,
  `src-tauri/src/files.rs`): every 10 s while there are unsaved changes,
  a lossless snapshot (IcyDraw plus the file's own SAUCE record, format
  name and render settings, since IcyDraw alone would drop them) goes to
  `Recovery/` in the app-data folder; saving, discarding or closing
  removes it, so what's left at launch is from a crash and is offered
  back as unsaved changes. Still to try in the app (the App Testing
  "Recent and recovery" items).
- [x] **Save formats:** `.ANS`, `.ASC`, `.XB` (with its font and
  palette), `.BIN`, `.ADF`, `.IDF`, `.TND`, all through icy_engine.
  **Done 2026-09-30.** `.TND` lost black text before its first color
  change until icy_engine's writer was fixed (icy_tools#188, fixed
  upstream as `da0d287`, pinned 2026-09-30).
- [x] **A save warns before it loses anything** (24-bit into `.BIN`, a
  custom font into `.ANS`, iCE into a format without it): a plain list,
  and the choice to go on. Phase 4 grows this into the full capability
  analysis. **Done 2026-09-30** in the Save dialog; a loss that makes the
  save impossible (odd width in BIN, not 80 columns in ADF, a character
  outside the font) disables it.
- [ ] **SAUCE editing** in the SAUCE panel: title, author, group, date,
  comments, flags (iCE, letter spacing, aspect ratio), font name. A
  save never appends a second record. **Done:** title, author, group
  and comments in the Save dialog; flags and font from the document; the
  file's own date kept (today for new art); one record, tested.
  **Left:** editing in the SAUCE panel itself.
- [x] **PNG export:** native pixel size, 8/9-px spacing, optional
  aspect correction, determinate progress over rows. **Done
  2026-09-30** (`stylus-core/src/export.rs`, File → Export PNG…, ⇧⌘E):
  the viewer's own pixels, streamed into the PNG 50 rows at a time so a
  long ANSI is never one image in memory, written beside the target and
  moved over it at the end. The dialog starts from the view's settings
  without changing them. Aspect correction resamples rows with area
  weights in integer maths (a flat area keeps its exact color, and any
  band size gives the same bytes). iCE follows the document. Still to
  try in the app (the App Testing "Export PNG" items).
- [ ] **Contrast lint:** the checker from
  `docs/ansiapps-color-contrast.md`, available on any document (theme
  mode enforces it in Phase 2).
- [x] **Round-trip check:** `scripts/roundtrip-corpus.sh` opens every
  corpus file, saves it in its own format and as `.ANS` and `.XB`,
  reopens each and compares pixels. **Done 2026-09-30:** on 181 files,
  353 saves identical, 9 differing only where the save warned (24-bit
  colors, custom fonts), none unexplained. It found that icy_engine's
  color optimizer rewrote look-alike cells (a black block on red as a
  blank) so a reopened ANSI ended early; saves are now cell-exact.

**Exit:** a first-time user can open a corpus file, change it, save it
as a new file with correct SAUCE, and export a PNG, with no data lost
that they weren't warned about.

---

## Phase 2: The theme workshop

**Goal:** the ANSIapps theme pack is drawn, checked and exported in
Stylus, and Diskette uses it. This is build step 4, unchanged from the
notes (section "The theme workshop") and `docs/ansiapps-theme.md`
("ANSI art assets").
**Release:** theme pack v1 in `theme/` (MIT), and a Stylus 0.x update.

- [ ] **Theme mode:** opens `theme/` (the `.XB` parts plus
  `manifest.json`) as a project. The canvas is locked to IBM VGA 8×16
  (VileR's font, rasterized at run time, never converted), 8-px
  spacing, square pixels, iCE on, and the 16 `--dos-*` colors.
- [ ] **Part editors:** 9-slice parts with every state side by side and
  a live stretch at several sizes; icons at 1×1, 2×1, 2×2 on every row
  color they can land on; graphics and animations (app marks, splash
  art, spinners), with animations using the Phase 3 timeline once it
  exists and single frames until then.
- [ ] **Live preview:** a sample ANSIapps screen built from the parts
  being edited.
- [ ] **Contrast checking:** ratio, rank and level in the color picker;
  a role per cell (text, meaningful graphic, decorative); failing cells
  marked and listed; the pack won't export with a failure.
- [ ] **Accessibility warnings while editing:** nothing flashes more
  than 3 times a second, and every animation has a rest frame.
- [ ] **`stylus-render`:** the command-line tool that renders the pack
  to PNG at 1× and 2× plus frame strips, writes `ansiapps-theme.css`,
  alt text and `aria-hidden` flags, re-runs the contrast check, and
  records the pack version.
- [ ] **Draw theme pack v1** and move Diskette onto it (Diskette-side
  work, from a session in `~/Coding/diskette`).

**Exit:** Diskette's build runs `stylus-render` and ships the rendered
pack; Stylus shows the pack's ANSI icons in place of SVG stand-ins.

---

## Phase 3: Make and animate

**Goal:** all five ways of making art, ANSImation, and the
reference-editor features the classic editors are known for.
**Release: Stylus 1.0** for macOS on itch.io.

### 3a. Converters (build step 5)

- [ ] **Image to ANSI:** the 36 ported converters are done. Still to
  come is Stylus's own converter from the notes: glyph sets (blocks
  only, shades, full CP437, half-blocks), OKLab color matching,
  optional dithering (ordered, Floyd–Steinberg), live preview, and the
  **theme-safe option** (contrast-list pairs only). Results open in the
  editor.
- [ ] **ASCII to ANSI:** colorize monochrome ASCII by rules (character
  class → color), gradients, region fills, or painting over it.
- [ ] **Text to ANSI:** TheDraw fonts (`.TDF`) for color, FIGlet
  (`.flf`) for monochrome, with users' own fonts loadable. Bundle only
  fonts with a clear license (open question in the notes).
- [ ] **Seeded random generation:** noise, cellular automata, symmetric
  logo patterns, plasma, gradients. Our own PRNG, integer math, and
  generator name, version, seed and settings in SAUCE comments (rule 5).

### 3b. Animation (build step 6)

- [ ] **Player:** classic ANSImations at a chosen baud rate (300 →
  28.8k, or unthrottled), with pause, step and scrub.
- [ ] **Timeline:** add, delete, duplicate and reorder frames,
  thumbnails, per-frame hold time, loop and ping-pong, onion skin.
- [ ] **Exports:** a classic `.ANS` ANSImation (frame diffing into the
  fewest cursor moves and writes, the intended baud rate in a SAUCE
  comment), IcyDraw's animation format if icy_engine writes it,
  animated GIF/APNG, PNG sequence, and the theme pack's frame strips.
- [ ] Theme-pack animations move onto the timeline.

### 3c. The rest of the reference-editor feature set

- [ ] **Layers:** add, delete, reorder, rename, hide, lock, merge,
  flatten, and per-cell transparency (IcyDraw), plus a reference-image
  layer for tracing that never saves into the art.
- [ ] **More selection transforms:** rotate 90°/180°, stamp, tile.
- [ ] **More drawing:** ellipse, shade and block gradients, color
  replace (foreground, background, both) over a selection or the
  document.
- [ ] **Fonts:** switch font per document, import/export custom fonts
  (`.F16`, `.PSF`, `.FNT`), a character-set browser, and the XBin/IDF
  embedded fonts editable.
- [ ] **Palette editing** for formats that carry one (XBin, ADF, IDF,
  TND), with import/export.
- [ ] **Command palette** (⌘K) and **custom keyboard shortcuts**, both
  from the command registry, since 1.0 has more commands than the menus
  show at a glance (moved from Phase 4 by `docs/ui-design.md`).
- [ ] Remaining save formats from Phase 1 that icy_engine didn't write
  (contribute the writers upstream where they fit).

### 3d. 1.0 polish

- [ ] macOS file associations for the art formats, and Quick Look
  previews of `.ANS` (a `docs/platform-parity.md` row each).
- [ ] **Diskette integration** (Diskette-side, optional): SAUCE in its
  catalog, "Open in Stylus" via the companion app, `.ANS` thumbnails
  from `stylus-core`.

**Exit:** every "Makes", "Animates" and "Saves and exports" item in the
notes works on macOS.

---

## Phase 4: Everywhere, and beyond

**Goal:** Stylus on every platform, then the tools no historical editor
had: Stylus as a preservation and research tool.

### 4a. Everywhere (build steps 7 and 8)

- [ ] **Web app at `stylus.ansiapps.com`:** the full app, not a trial
  (the notes' parity table). WASM core in the browser, files read and
  saved locally only, Cloudflare Pages static hosting, installable and
  offline (PWA). Solve rayon on wasm32 first (the maintainer offered to
  help).
- [ ] **Windows and Linux** builds on GitHub Actions, filling in
  `docs/platform-parity.md`.

### 4b. Beyond (new; order to be set when 4a ships)

- [ ] **Format capability analysis:** a capability matrix generated
  from the code (not hand-kept), a loss report before any save or
  convert, and a preview of the degraded result.
- [ ] **Emulation profiles / historical accuracy mode:** a profile
  fixes palette, font, screen size, escape-sequence support and timing
  (DOS ANSI.SYS, VGA, iCE, XBin, PCBoard, a modern terminal), and the
  editor can be limited to one.
- [ ] **ANSI playback lab:** step through a file byte by byte or
  sequence by sequence, with the raw sequence, cursor, attributes and
  screen state shown at each step.
- [ ] **Preservation diagnostics:** malformed or unsupported sequences,
  missing fonts, truncation, SAUCE problems, as a report per file or
  folder.
- [ ] **Compatibility test suite:** the Phase 1 round-trip check grown
  into import → render → export → reimport comparisons across formats.
- [ ] **Parser fuzzing** of icy_engine's loaders (found crashes go
  upstream). Art files come from strangers, so this is a security
  item too.
- [ ] **More formats, upstream first:** RIPscrip viewing, UTF-8 ANSI
  with 256/24-bit color, mIRC color codes, HTML export, Sixel export.
- [ ] **Modern text art:** Unicode block elements, quadrants,
  sextants and braille as drawing modes, for modern terminal output.
- [ ] **Effects:** palette cycling, and non-destructive effects over a
  frame range (wipes, dissolves, type-on reveals), all deterministic.
- [ ] **Power use:** batch conversion and a headless CLI (the
  `stylus-render` binary grown), macro recording.
- [ ] **The Lab workspace** (`docs/ui-design.md`) holding the playback
  lab, source/render split and format inspector, and responsive
  drawers for small windows and the web app.

**Exit:** 4a has one; 4b is an open list worked from the top.

---

## Not planned

Items in the surveys Stylus won't build, and why. Changing any of these
needs a decision recorded in the notes first.

| Item (survey) | Why not |
|---|---|
| **Network collaboration**, shared canvases, remote cursors, chat (ChatGPT §26, 3C; Gemini 2.5) | Rule 1: no server, and no art leaves the machine. |
| **AI-assisted conversion and restoration** (ChatGPT §63, §64) | Rule 6: every way of making art is a deterministic algorithm. The notes allow a revisit only with licensed training data and a local model. |
| **Clones of other editors' interfaces** (TheDraw-, ACiDDraw-, iCE-style UI; ChatGPT §46) | Stylus has one look, the ANSIapps theme (with modern as the option). The F-key character sets keep the classic workflow. |
| **A terminal-native editor** (Gemini 3.6, after Durdraw) | Stylus is a Tauri and web app. The headless CLI (Phase 4) covers scripting. |
| **Audio-synced timeline** with tracker/MIDI files (Gemini 4.4) | Far from the art tool's job, and a player would bring its own license questions. |
| **Byte-for-byte lossless editing** of untouched regions, and a provenance graph (ChatGPT §55–56, §66) | Rule 4 already keeps the original safe (opening never changes it). Phase 4's loss report and round-trip tests cover what matters for users. |
| **Automatic historical reconstruction** (guessing editor, era, terminal; ChatGPT §62) | Guesswork presented as fact. Phase 4's diagnostics report what the file actually says. |
| **Hybrid RIPscrip vector/ANSI editing** (Gemini 4.2) | Viewing RIPscrip is in Phase 4; editing vectors is a different app. |
| **Emoji-aware mode, combining characters** (ChatGPT §29) | Not ANSI art; revisit if the modern-text-art modes call for them. |

## Decisions made here

For the notes' next update (a session in `~/Coding/diskette`):

- The eight build steps are grouped into these four phases; Stylus 1.0
  is the end of Phase 3 (macOS), the first public beta the end of
  Phase 1.
- Phase 1 adds, beyond the notes' step 3: a loss warning on save,
  autosave for recovery, the round-trip check, and the requirement that
  the document model carries layers and frames from the start.
- Layers, TheDraw fonts, palette editing and the extra transforms are
  Phase 3.
- Phase 4b is new, drawn from the surveys' research phases.
- From `docs/ui-design.md` (2026-09-30): the command palette and custom
  shortcuts are Phase 3, not 4, and Phase 3 gains a reference-image
  layer.
- The "Not planned" list above.
