# Changelog

## 0.3.0 (2026-09-30, unreleased)

- **Editing begins (Phase 1)** (2026-09-30): the Viewer tab is now the Art
  workspace, where opened art can be edited and new art made.
  - New… (80×25, 80×50, 132×25, 160×25 or any size, iCE on or off).
  - Type at a cursor (arrows, Home/End, Page Up/Down, Backspace, Delete;
    Enter or Down on the last row adds a row), the Pencil (right-drag
    erases), Pick (or Option-click), and painting characters and colors,
    colors only or characters only.
  - Ten F-key character sets on F1–F10 and a clickable strip, a
    256-character picker, 16 color swatches, a status bar.
  - Undo and redo by stroke (⌘Z, ⇧⌘Z).
  - Save and Save As in ANSI, plain text, XBin, BIN, Artworx, iCE Draw
    and TundraDraw, through icy_engine. The dialog lists what the format
    would lose before saving, and won't save what it can't hold. SAUCE
    title, author, group and comments are written in one fresh record,
    keeping the file's date. Save asks the first time it would replace a
    file; a replaced file is written beside it and moved over it.
  - Opening, New or closing the window with unsaved changes asks first.
- **The drawing tools and selection** (2026-09-30): Eraser, Line,
  Rectangle (outline or filled), Box (single or double lines), the
  Half-block brush (half-cell pixels, right-drag in the background color),
  and flood Fill join the Pencil. Shapes preview as the real art while
  dragged and undo in one step. Select drags a rectangle (or Shift+arrows,
  ⌘A), and the selection can be cut, copied, pasted (opaque or
  transparent, into any open document), moved by dragging, flipped (with
  ▌▐, ▀▄ and the box corners mirrored), filled and cleared. Single-letter
  tool keys work while not typing; Esc leaves Type.
- **Menu bar** (2026-09-30): File, Edit, Draw, Select, Colors, View and
  Help, drawn from one command registry that also runs every shortcut
  and fills Help → Keyboard Shortcuts. Option plus a menu's letter opens
  it, and arrows move through it. The Art and Make workspaces and the
  gear sit at the bar's right; the editor's toolbar went into the menus
  (zoom and the render settings in View, iCE in Colors, the Amiga fonts
  in View for text files). The art's name, with • when unsaved, is in
  the window title and the status bar. X swaps the colors.
- **Amiga ASCII** (2026-09-30): text files without SAUCE get a Font menu
  with the Amiga fonts, remembered per file type.
- **UTF-8 text files** (2026-09-30) show their block characters (▀ ▄ █),
  which came out blank before.
- **Saving tested on real art** (2026-09-30): `scripts/roundtrip-corpus.sh`
  saves every corpus file again and compares pixels. On 181 files: 353
  saves identical, 9 differing only where the save warned. It found
  icy_engine's color optimizer dropping look-alike rows from ANSI, so
  saves are now cell-exact, and a TundraDraw writer bug, written up in
  `docs/upstream/icy_engine-tundra-initial-color.md` and sent upstream as
  icy_tools#188.
- The "77 rows" 162-column file is explained: its SAUCE asks for the EGA
  8×14 font, which Stylus uses and ansilove doesn't.

- **Roadmap** (2026-09-30): `docs/roadmap.md` consolidates the two
  ANSI-editor feature surveys with the design notes into a four-phase
  rollout (open and draw, theme workshop, make and animate for 1.0,
  everywhere and beyond), and lists the survey features Stylus won't
  build and why.
- **UI design** (2026-09-30): `docs/ui-design.md` adapts the ANSI editor
  UI survey to Stylus's phases and the ANSIapps theme: the Art, Make,
  Theme and Lab workspaces, the F-key strip, one Problems list, and a
  native macOS menu bar beside the in-window one. The roadmap moves the
  command palette and custom shortcuts to Phase 3 and adds a
  reference-image layer.

- **In-development warning** (2026-09-30): on first run, before any
  window, a native dialog says Stylus is still in development. OK opens
  the app and is remembered (a marker file in the app-data folder);
  I'll Be Back. quits and asks again next launch. The main window is now
  created only after that (`src-tauri/src/first_run.rs`, after
  Diskette's).

- **Image to ANSI** (2026-09-30): a new tab. Drop an image on the window
  (or choose one) and 36 open-source image-to-ANSI converters each turn
  it into an 80-column `.ANS`, shown side by side, each labelled with its
  project, repository, license and the settings used. Save one, or all
  into a folder (existing files are never replaced).
  - The converters are Rust ports in `stylus-core/src/convert/`, all
    under licenses that allow closed-source commercial use: KOAN.ansi,
    img2ansi (Wes Brown), libcaca, two png2ansis, ansify, ansir, CLImage,
    TerminalImageViewer, viuer, catimg, terminal-image, two imgcats,
    termpix, image-to-ansi, aimg, img2ansi (John McCabe, Jakob Westhoff),
    ansipx, asciimatics, term-image, Rich Pixels, img2txt, img2ansi
    (Bryan Matsuo), hiptext, ansize, ascii-image-converter, ASCII Magic,
    img2ansi (Lukas Beranek), tapciify, image-to-ascii, img_to_txt,
    chromatic, ANSI-art and ransid. Two carry a whole resizer:
    ImageMagick's (`magick.rs`) and stb_image_resize2's (`stbir.rs`).
  - Every one matches its original cell for cell on the test images
    except ansify (99.8%+, its k-d tree breaking ties its own way). The
    survey, exclusions and results are in
    `docs/image-to-ansi-converters.md`.
  - About lists every converter's and ported library's license in full.

- **icy_tools upgraded to `2e4e2b1`** (2026-09-30), which has #187 as
  merged: DEL shows as ⌂ and 24-bit backgrounds stay put under iCE.
  BEL stays a control code, as icy_engine's maintainer chose, so files
  with BEL differ from ansilove's •. Rechecked on 128 files: all match
  except that BEL file and one 162-column wrapped file (older, still
  unexplained).
- **stylus-core now builds for WebAssembly** with icy_engine's default
  features: the maintainer removed icy_net, zstd's C code and unrar
  from icy_engine's tree upstream (in place of our feature-flag
  proposal, icy_tools#186).

## 0.2.0 (2026-09-29, unreleased)

- **Viewer** (2026-09-29, build step 2): open art with Open… or by
  dropping it on the window. Stylus only reads the file.
  - Every format icy_engine loads: `.ANS`, `.ICE`, ASCII (`.ASC`, `.NFO`,
    `.DIZ`, `.TXT`), `.BIN`, `.XB`, `.ADF`, `.IDF`, `.TND`, `.PCB`,
    `.AVT`, IcyDraw, Renegade, CtrlA, PETSCII, ATASCII and REXPaint.
  - **SAUCE always shown**: every field, decoded from CP437, plus what's
    on screen (format, size, font, cell and image size). Files without
    SAUCE say so.
  - **9-px spacing, iCE colors and aspect ratio** switches, starting
    from the file's SAUCE. Without iCE, blinking text really blinks, once
    a second, and holds still with Reduce motion on. Zoom: fit width,
    100%, 200%, 300%.
  - Rendered through icy_engine's renderer in bands, with a progress bar
    counting rows, so long ANSIs render completely. ANSI and other parsed
    formats end at their last line instead of a padded 25-line screen.
  - `scripts/compare-ansilove.sh` compares every file in a local corpus
    with ansilove's render, pixel by pixel. On Stylus-made test files,
    Stylus matches ansilove exactly, apart from an ansilove bug in `.BIN`
    blink mode.
  - Fixed: for files without iCE in SAUCE, the iCE switch showed on while
    the art was drawn with blink, and ticking it did nothing.
  - Fixed: `.ASC` and `.TXT` files holding ANSI color codes (common in
    the scene) showed the codes as characters. They're read as ANSI now.
  - Checked against ansilove on 162 files from six Sixteen Colors packs:
    all but 7 match pixel for pixel. The 7 came from icy_engine's
    handling of BEL, DEL and iCE with 24-bit backgrounds, raised in
    [icy_tools#187](https://github.com/mkrueger/icy_tools/pull/187).

## 0.1.0 (2026-09-29, unreleased)

- **Scaffold** (2026-09-29, build step 1 in Diskette's
  `docs/stylus-notes.md`): Tauri 2 + React/TypeScript, bundle ID
  `com.ansiapps.stylus`, dev port 1450, MIT `LICENSE` for the app and
  `theme/`.
  - **`stylus-core`**, a Tauri-free Rust crate on icy_tools, pinned to
    revision `b85e565`. So far it reports its version and opens art from
    bytes through icy_engine (ANSI and SAUCE covered by tests).
    icy_engine didn't build for WebAssembly then (icy_net's tokio and
    rustls, unarc-rs, and zstd's C code); we proposed feature flags in
    [icy_tools#186](https://github.com/mkrueger/icy_tools/pull/186), and
    the maintainer fixed it upstream instead (see 0.2.0).
  - From Diskette: both themes (**ANSIapps is the default**, key
    `stylus.theme`, modern in the gear menu), the launch splash and
    `StartupScreen`, `ProgressBar` with the completion estimate,
    `runActivity`/`ActivityStatus`, `Dialog`, the gear menu, the dev-only
    App Testing checklist, the design tokens, `build-release.sh`, and the
    theme and contrast docs.
  - Stylus's accent is a fuchsia that maps to light magenta in the
    ANSIapps theme (provisional). In the ANSIapps theme a dialog's primary
    button is black on light magenta (8.0:1), not white on green (3.11:1,
    which fails).
  - About credits icy_tools and VileR's VGA font.
  - `scripts/license-scan.py` for every lockfile change.
