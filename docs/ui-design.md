# Stylus UI design

**Decided 2026-09-30.** The screen Stylus grows into across the four
phases in `docs/roadmap.md`. It adapts `ansi-editor-ui-design-chatgpt.md`
(a UI proposal for a generic ANSI editor, kept as reference) to Stylus:
its roadmap, its rules in `CLAUDE.md`, the ANSIapps theme
(`docs/ansiapps-theme.md`) and the contrast list
(`docs/ansiapps-color-contrast.md`). Where they disagree, those win
over this doc, and this doc wins over the survey.

## What we keep from the survey, and what changes

Kept, because it fits Stylus as is:

- **The canvas dominates.** Tools on the left, one context panel on the
  right, a character and color strip and a status bar along the bottom,
  and a timeline only when there's animation.
- **Progressive disclosure.** Drawing an 80×25 screen never shows the
  Phase 3 or 4 machinery. Specialist views (fonts, SAUCE, formats,
  diagnostics) are panels and workspaces opened on demand.
- **One document model, many views.** What you draw (the document), how
  you draw it (tool, character, colors, layer) and how it's saved (the
  format) are separate, so changing the save format never changes the
  document until you save.
- **Layers and frames are separate panels** over one document, not one
  tree.
- **Both a cell editor and a painter** (half-blocks) on the same
  document.
- **Keyboard first,** with the F-key character sets.
- **Preview before commit** for shapes and, later, transforms and
  recolors.
- **Clickable warnings** that highlight the cells they're about.

Changed for Stylus:

- **Two themes, ANSIapps first.** Every piece of UI is designed on the
  ANSIapps theme's 8×16 character grid and 16 colors first, and must
  work in full in the modern theme too. The survey assumes one generic
  look.
- **The theme workshop** (Phase 2) is a workspace of its own. The survey
  doesn't have one.
- **One Problems list** holds save-loss warnings, contrast failures,
  animation accessibility and (Phase 4) diagnostics, instead of the
  survey's separate diagnostics, compatibility and preservation panels.
- **Dropped:** collaboration, AI tools, historical UI presets
  (TheDraw-like and so on), a terminal UI, user-level presets (Beginner,
  Power User…), provenance and the art library. See "Not adopted".
- **Phased:** each part below says which phase adds it. Nothing is
  built before its phase, but Phase 1 lays out the window so later
  parts slot in without moving anything.

## Principles

1. **The art's colors, the theme's colors.** The canvas always shows
   the art's own colors (rule 7). Everything around it (menus, panels,
   swatch frames, the status bar) follows the theme, with every
   text/background pair from the contrast list. A swatch is art; its
   label and frame are UI.
2. **Sized in cells.** In the ANSIapps theme the font is 16px only, so
   every bar, rail and panel is a whole number of 8×16 cells. The modern
   theme uses the same proportions.
3. **Nothing moves the art.** Opening a panel, the timeline or a
   workspace resizes the canvas area; it never scrolls or re-zooms the
   art.
4. **Every command lives in one registry** (name, shortcut, menu
   placement, enabled state, phase). Menus, shortcuts, the tool rail and
   the command palette all read it, so a command is added once.
5. **Feedback for every action** through `runActivity` and
   `ProgressBar`: PNG export counts rows, conversion counts cells, an
   animation export counts frames, a theme-pack render counts parts.
6. **The UI never names a repo file** (`docs/`, `CLAUDE.md` and so on).
   Help and warnings say what's needed in the UI itself.

## The window

Art workspace, Phase 1, ANSIapps theme (light-gray menu bar, blue
desktop, double-line panel frames):

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ File  Edit  Draw  Select  Colors  View  Help               Art Make  ⚙  │ menu bar + workspaces + gear
├──────────────────────────────────────────────────────────────────────────┤
│ LOGO.ANS •  │ untitled-1  │                                              │ document tabs (• = unsaved)
├───┬──────────────────────────────────────────────────┬───────────────────┤
│ ✎ │                                                  │╔═ Character ═════╗│
│ ╱ │                                                  │║ CP437 grid      ║│
│ □ │                                                  │╚═════════════════╝│
│ ▣ │                  CANVAS                          │╔═ Colors ════════╗│
│ ▀ │            (the art's own colors)                │║ 16 swatches     ║│
│ T │                                                  │╚═════════════════╝│
│ ⬚ │                                                  │  Character · Colors│
│ ◉ │                                                  │  SAUCE · Problems │ panel tabs
├───┴──────────────────────────────────────────────────┴───────────────────┤
│ F1 ░ F2 ▒ F3 ▓ F4 █ F5 ▀ F6 ▄ F7 ▌ F8 ▐ F9 ■ F10 ·  │ set 5/10 │ FG ■ BG ■ │ the strip
├──────────────────────────────────────────────────────────────────────────┤
│ Pencil │ █ 219 │ FG 15 BG 1 │ 42,18 │ 80×50 │ ANSI · iCE · 9px │ 200% │ status bar
└──────────────────────────────────────────────────────────────────────────┘
```

Sizes (ANSIapps; the modern theme matches them in px):

| Part | Size | Notes |
|---|---|---|
| Menu bar | 1 row | Light-gray bar, black text, access letters blue (rank 23; Turbo Vision's red on light gray fails). Open menus highlight green with black text |
| Document tabs | 1 row | Hidden with only one document open |
| Tool rail | 4 columns | 2×1-cell icons (16×16 px) with a cell of margin either side |
| Context panel | 34 columns | Resizable in whole columns, 26 minimum; closable |
| The strip | 1 row | Always visible in the Art workspace |
| Timeline (Phase 3) | 6 rows | Collapsed to its 1-row header unless the document has frames |
| Status bar | 1 row | Blue, light-gray text (rank 23) |

The gear menu stays where every ansiapps app has it (theme toggle, App
Testing, About) and stays identical to Diskette's.

### Menus and the macOS menu bar

The in-window menu bar is part of the look (Turbo Vision) and is what
the web app and Windows/Linux get. **On macOS the same commands are also
in the native menu bar**, built from the same registry, because Mac
users expect ⌘ shortcuts and the Services, Window and Help menus there,
and VoiceOver reads it. (A `docs/platform-parity.md` row when it lands.)

Menus, by phase (P1–P4):

| Menu | Items |
|---|---|
| **File** | New… (size, format, font), Open…, Open Recent, Close, Save, Save As…, Export PNG…, Document Properties (P1); Export animation… (P3); Batch convert… (P4) |
| **Edit** | Undo, Redo, Cut, Copy, Paste, Paste transparent, Delete, Select All (P1) |
| **Draw** | Pencil, Eraser, Line, Rectangle, Filled rectangle, Box (single/double), Fill, Half-block brush, Text, Eyedropper (P1); Ellipse, Stamp, auto-joining box lines, Recolor (P3) |
| **Select** | Select All, Deselect, Flip horizontal/vertical, Fill selection, Clear (P1); Rotate, Tile, Save as stamp (P3) |
| **Colors** | Swap foreground/background, iCE colors on/off, Paint: character + color / color only / character only (P1); Palette editor (P3) |
| **Animation** (P3) | Show timeline, New/Duplicate/Delete frame, Play/Pause, Step, Onion skin, Playback: frame timing / baud rate |
| **View** | Zoom in/out, Actual size, Fit, Grid, 9-px spacing, Aspect correction, Preview (hide overlays), Show panel ▸ (P1); Layers, Reference image (P3); Lab ▸ (P4) |
| **Help** | Keyboard shortcuts, About Stylus (P1) |

A menu gets an item only when its phase ships; no greyed-out
placeholders for future features.

## Workspaces

The current **Viewer | Image to ANSI** tabs become the workspace
switcher at the right of the menu bar. Each workspace is a layout, not a
separate app; they share documents, the command registry and the
Problems list.

| Workspace | Phase | What it is |
|---|---|---|
| **Art** | 1 | The editor above. Opening a file lands here (it replaces the Viewer tab) |
| **Make** | 1 (Image to ANSI as today), 3 (the rest) | Ways of making art that aren't drawing: Image, ASCII, Generate. Every result opens as a new document in Art |
| **Theme** | 2 | The theme workshop, shown only once a theme pack is open |
| **Lab** | 4 | Format inspector, playback lab, diagnostics |

Text to ANSI isn't in Make: it's the Art workspace's **Text tool** with
a TheDraw or FIGlet font chosen (P3), so the lettering lands on the
canvas where you click.

## Art workspace, part by part

### The canvas

- **Viewing and editing are one state.** A file opens ready to edit,
  with the same render settings as today's viewer (9-px, iCE, aspect,
  zoom, real blink). Nothing is written until Save (rule 4); the tab's
  • shows unsaved changes.
- **Overlays:** cursor (a blinking block in the theme, steady under
  reduced motion), selection marquee, grid (View menu), and the
  rubber-band preview of the shape being drawn. **Preview** (View menu,
  or holding a key) hides them all, which is also how a finished piece
  is checked.
- **Blink** plays as it does in the viewer (two frames a second, never
  under reduced motion). While editing, blinking cells stay drawn so
  they can be seen and picked.
- **Canvas background** outside the art is the theme's desktop, with a
  1-cell black margin so the art's own black edge stays visible.

### Tool rail

Only the everyday tools, in this order: Pencil, Line, Rectangle, Box,
Half-block brush, Text, Select, Fill, Eyedropper (P1). Eraser, filled
rectangle and the Phase 3 shapes are flyouts on their rail neighbours
(hold, or right-click). Undo and Redo stay in Edit and on ⌘Z/⇧⌘Z rather
than on the rail.

**The rail's icons are the first theme-pack icons.** They start as SVG
stand-ins (the theme's rule), and Phase 2 redraws them as 2×1-cell ANSI
icons in Stylus itself: the first real test of the icon editor.

### The strip (character, colors, F-keys)

The survey's character/color strip, made the TheDraw way: the current
**F-key set** shown as ten keys and their characters, the set number
(click or ⌃← ⌃→ to change), and the foreground/background swatches.
Clicking an F-key cell types its character, which matters on Mac
laptops where F-keys need fn. Double-clicking one edits the set.

In Phase 3 the right end also shows the current **stamp** when one is
active.

### Context panel

Tabs, one open at a time, the last choice remembered per workspace:

| Tab | Phase | Contents |
|---|---|---|
| **Character** | 1 | The full font as a 16×16 grid in the document's own font, the selected character's CP437 code and Unicode name, entry by code. Font switching and import (P3) |
| **Colors** | 1 | 16 swatches (or the document's palette), FG/BG, iCE state; in P3 palette editing for formats that carry one |
| **SAUCE** | 1 | Today's `SaucePanel`, made editable: title, author, group, date, comments, flags, font |
| **Properties** | 1 | Changes with context: the document (size, format, font, spacing), the cell under the cursor (character, code, colors, attributes), or the selection (size, character and color counts) |
| **Problems** | 1 | See below |
| **Layers** | 3 | Add, reorder, rename, hide, lock, merge; a reference-image layer that never saves into the art |

SAUCE is a tab, not permanent UI, but it's always one click away (rule:
SAUCE is always shown and editable).

### Problems

One list for everything Stylus wants you to know about the document.
Each row names the problem in plain words, counts the cells, and
selecting it highlights them on the canvas.

| Source | Phase |
|---|---|
| **Save loss:** what the chosen format can't hold (24-bit colors, a custom font, iCE, layers). Also shown in the Save As dialog before writing | 1 |
| **Contrast lint:** text cells off the 32 passing pairs, meaningful graphics under 3:1 (the lint is on per document; theme mode forces it on) | 1 |
| **Animation accessibility:** flashing faster than 3 times a second, no rest frame | 2 (theme parts), 3 (every animation) |
| **Diagnostics:** malformed or unknown sequences, truncation, SAUCE problems | 4 |

The tab shows a count (e.g. `Problems 3`); it never opens itself over
the user's chosen tab.

### Status bar

Always: tool, current character and code, FG/BG numbers, cursor cell,
document size, format with its flags (iCE, 9-px, 24-bit), zoom. When
relevant: selection size, frame `3/12` and timing (P3), the Phase 4
emulation profile.

The cell under the pointer or cursor is also given to screen readers on
request (a shortcut reads it out through an `aria-live` region), so the
canvas isn't a black box to VoiceOver.

### Timeline (Phase 3)

A bottom workspace, collapsed to its header until the document has more
than one frame (File → New with frames, or Animation → New frame):

```text
│ ▶ ❚❚ ◀│ │▶  + ⧉ ✕ │ Frame 3/12 │ hold 100 ms │ Onion ○ │ Timing: frames ▾ │
│ ▣ │ ▣ │ ▣ │ ▣ │ ▣ │ ▣ │ ▣ │ ▣ │ ▣ │ ▣ │ ▣ │ ▣                           │
```

- Thumbnails are rendered by `stylus-core` like `ArtThumb` today.
- **Timing: frames** (per-frame hold in ms, for GIF/APNG and theme-pack
  strips) or **baud** (300 to 28.8k, or unthrottled, for classic
  `.ANS` ANSImations). The survey's two timing models, as one control.
- Playback never starts by itself, and under reduced motion shows the
  rest frame until Play is pressed. Anything playing over 5 seconds has
  Pause (it always does here).
- Layers stay in the context panel; the timeline shows frames only.

## Make workspace

Phase 1 keeps today's Image to ANSI screen as it is (36 converters side
by side). Phase 3 turns it into three tabs down the left:

- **Image:** Stylus's own converter on top (glyph set, OKLab matching,
  dithering, **theme-safe**, a live preview updating as settings
  change), with the 36 ports below as today.
- **ASCII:** load or paste ASCII, colorize by rules, gradients or
  regions, with a live preview.
- **Generate:** pick a generator, its settings, and the **seed** in a
  field that can be copied, typed and re-rolled. The result shows its
  generator name and version, and they go into SAUCE comments.

Every tab ends with **Open in editor**, which makes a new untitled
document in Art. Conversion shows determinate progress over cells.

## Theme workspace (Phase 2)

Opened by File → Open theme pack…, or by opening `theme/`. The canvas
is locked to the theme's setup (VGA 8×16, 8-px, square pixels, iCE, the
16 colors), and the status bar says so.

```text
┌─────────────┬─────────────────────────────────────────┬──────────────────┐
│ ╔═ Parts ══╗│                                         │╔═ Part ═════════╗│
│ ║ Buttons  ║│                                         │║ button-primary ║│
│ ║  default ║│         CANVAS (the part)               │║ state: hover ▾ ║│
│ ║ >primary ║│                                         │║ slices 1,1,1,1 ║│
│ ║  danger  ║│                                         │║ alt text       ║│
│ ║ Dialog   ║│                                         │║ role: text ▾   ║│
│ ║ Icons    ║│                                         │╚════════════════╝│
│ ║ Spinners ║├─────────────────────────────────────────┤ Part · Colors ·  │
│ ╚══════════╝│ STATES  normal hover pressed focus dis. │ Problems         │
│             │ STRETCH  [  OK  ] [   Cancel   ] [..]   │                  │
└─────────────┴─────────────────────────────────────────┴──────────────────┘
```

- **Parts list** (left, replacing nothing: the tool rail stays beside
  it) from `manifest.json`, grouped by kind, with a mark on any part
  with problems.
- **Below the canvas,** for a 9-slice part: every state side by side,
  and the part stretched live to several sizes. For an icon: the icon on
  every row color it can land on (blue, cyan selection, light gray,
  brown, black).
- **Part tab:** state, slice insets, alt text or decorative, default
  cell role; per-cell roles are painted with a Role tool.
- **Colors tab in theme mode** shows each swatch's ratio, rank and level
  (AAA / AA / 3:1 only / fail) against the current background.
- **Preview** (a toggle on the workspace) swaps the canvas for a sample
  ANSIapps screen built from the pack as edited: a panel, a dialog, a
  menu, a progress bar.
- **Export pack** runs `stylus-render` with determinate progress over
  parts and refuses while Problems has a contrast failure.

## Lab workspace (Phase 4)

The survey's format inspector, ANSI inspector and source/render split,
kept together and out of the everyday UI:

- **Source | Render split:** the file's bytes as sequences on the left,
  the render on the right; selecting either highlights the other.
- **Playback lab:** step byte by byte or sequence by sequence, with the
  cursor, attributes and screen state at each step.
- **Format and capability:** what the file is, what each save format
  would keep or lose (the survey's compatibility bars, as a table with
  words, not only bars).
- **Emulation profile:** the survey's "target profile", chosen here and
  shown in the status bar; it can limit the editor to that target.
- Diagnostics feed the shared Problems list.

## Keyboard

- **Mac:** ⌘ for the standard commands (⌘N, ⌘O, ⌘S, ⇧⌘S, ⌘Z, ⇧⌘Z,
  ⌘X/C/V, ⌘A, ⌘+/⌘−, ⌘0). Ctrl on Windows and Linux, from the same
  registry.
- **Canvas:** arrows move the cursor, Shift+arrows select, Home/End,
  Page Up/Down, Insert toggles overwrite/insert, typing types.
- **F-keys** type the current set's characters; ⌃← ⌃→ change set.
- **Single-letter tool keys** (P for pencil, L line, …) only while the
  canvas isn't in typing mode, so typing text never switches tools.
  Esc leaves typing mode.
- **Help → Keyboard shortcuts** lists everything from the registry.
- Custom shortcuts and the command palette: see Phase 3 below.

## Both themes

- **ANSIapps:** as drawn above. Panels are blue with light-cyan double
  frames and centered titles; menus and dialogs light gray with black
  text; the selected tab, tool and list row are the cyan bar with black
  text; focus is the dashed white outline. Panels open and close
  instantly (no easing). Status marks (unsaved •, problem ■) are CP437
  characters.
- **Modern:** the same layout and sizes on the shared tokens, accent
  fuchsia. Nothing in the modern theme may be missing or only reachable
  in ANSIapps.
- **Stylus's accent** (light magenta in ANSIapps) marks the current
  tool and the active workspace, on black or blue only (ranks 10 and 28).
- Every new piece is checked in both themes (App Testing).

## Small windows and the web (Phase 4)

- The window's minimum is 80×30 cells (640×480 px) of UI.
- Below 120 columns the context panel becomes a drawer over the canvas
  (a button on the rail opens it); below 90 the tool rail folds into the
  strip.
- The web app uses exactly this layout, with the in-window menu bar as
  its only menu bar. Touch gets the drawers and a larger strip; drawing
  with a finger or pen uses the same tools.

## Phase by phase

| Phase | The UI gains |
|---|---|
| **1** | Menu bar (and the macOS native one), workspace switcher (Art, Make), document tabs, tool rail, the strip, context panel (Character, Colors, SAUCE, Properties, Problems), status bar, the command registry, Save As loss warning. The Viewer tab goes; `ArtViewer` becomes the Art canvas |
| **2** | Theme workspace, Role tool, contrast info in Colors, the rail's icons redrawn as theme-pack icons |
| **3** | Timeline, Animation menu, Layers tab, reference image, Make's Image/ASCII/Generate tabs, TDF/FIGlet in the Text tool, stamps, palette editing, **command palette** (⌘K, from the registry) and **custom shortcuts**, since 1.0 has more commands than the menus show at a glance |
| **4** | Lab workspace, emulation profile in the status bar, diagnostics in Problems, responsive drawers for the web app and small windows |

## Not adopted

| Survey section | Why not |
|---|---|
| **Collaboration workspace** (§29) | Roadmap "Not planned": rule 1, no art leaves the machine. |
| **AI tools** (§46) | Rule 6. |
| **Historical UI presets** (TheDraw-like, ACiDDraw, iCE; §48–49) | Roadmap "Not planned". The F-key strip and keyboard cover the classic workflow in Stylus's own look. |
| **Terminal UI** (§37) | Roadmap "Not planned". |
| **User presets** (Beginner, ANSI Artist, Power User, Preservationist, Researcher; §35) | Disclosure comes from what you open (panels, workspaces), remembered per workspace. Presets would hide features behind a choice people make before they know the app. |
| **Authentic / Extended / Modern mode switch** (§24) | Phase 1's save-loss warning and Phase 4's emulation profile cover it without a global mode that changes what the tools do. |
| **Provenance graph, preservation package** (§25–26) | Roadmap "Not planned". |
| **Art library / art-pack browser** (§30) | Browsing and cataloguing collections is Diskette's job; Stylus is reached from it through "Open in Stylus" (roadmap 3d). |
| **Brush library and pattern editor** (§11) | Phase 3's stamps (a selection saved as a brush) cover the need; a library can come later if people ask. |
| **Split views and multi-document comparison** (§52 of the roadmap survey) | Not in the roadmap; document tabs cover several documents. |

## Decisions for the notes

For the next update of `stylus-notes.md` (a session in
`~/Coding/diskette`):

- The layout above, sized in 8×16 cells, designed on ANSIapps first.
- Viewing and editing are one state; the Viewer tab is replaced by the
  Art workspace in Phase 1.
- Workspaces: Art, Make, Theme, Lab.
- One Problems list for loss, contrast, animation accessibility and
  diagnostics.
- On macOS, a native menu bar alongside the in-window one, from one
  command registry.
- The command palette and custom shortcuts move from roadmap Phase 4 to
  Phase 3.
- A reference-image layer is added to Phase 3.
