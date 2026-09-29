# Changelog

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
    all but 7 match pixel for pixel. The 7 come from two icy_engine bugs
    (BEL and DEL shown as nothing instead of • and ⌂; iCE turning 24-bit
    backgrounds gray), fixed in
    [icy_tools#187](https://github.com/mkrueger/icy_tools/pull/187).

## 0.1.0 (2026-09-29, unreleased)

- **Scaffold** (2026-09-29, build step 1 in Diskette's
  `docs/stylus-notes.md`): Tauri 2 + React/TypeScript, bundle ID
  `com.ansiapps.stylus`, dev port 1450, MIT `LICENSE` for the app and
  `theme/`.
  - **`stylus-core`**, a Tauri-free Rust crate on icy_tools, pinned to
    revision `b85e565`. So far it reports its version and opens art from
    bytes through icy_engine (ANSI and SAUCE covered by tests).
    **icy_engine doesn't build for WebAssembly yet** (icy_net's tokio
    and rustls, unarc-rs, and zstd's C code); icy_parser_core and
    icy_sauce do. Proposed upstream fix: optional `net` and `archives`
    features for icy_engine,
    [icy_tools#186](https://github.com/mkrueger/icy_tools/pull/186).
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
