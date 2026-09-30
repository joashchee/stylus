# Changelog

## 0.3.0 (2026-09-30, unreleased)

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
