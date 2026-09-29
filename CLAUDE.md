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

## Current milestone: build step 2, the viewer (built, being checked)

Step 1 (scaffold) is done. The WASM fix is upstream as icy_tools#186
(see "Open before step 2").

Step 2 is built (2026-09-29): open by dialog or drag and drop, every
format icy_engine loads (`OPEN_EXTENSIONS` in `stylus-core/src/lib.rs`),
SAUCE always shown, 9-px spacing, iCE (and real blinking without it),
aspect correction and zoom. **Left before it's done:** the App Testing
pass on real art, and the libansilove comparison
(`scripts/compare-ansilove.sh`) against libansilove. **Checked
2026-09-29** on six Sixteen Colors packs (162 files, 1995–2024, in the
scratch folder, never the repo): every file renders, and all but 7 match
ansilove's pixels exactly (the rest differ only in the intended ways
listed in the script's header). The 7 are two icy_engine bugs, both
in `parser_sink.rs`, proposed upstream as icy_tools#187 (see "Open
before step 2"): BEL (0x07) and DEL (0x7F) are dropped as control
codes where art files mean the • and ⌂ glyphs, and iCE turns a 24-bit
background with the blink bit into palette color 8. With the fix, all
162 match. An ansilove bug also shows up in `.BIN` blink mode (the
script's header; fix proposed as libansilove#28). Amiga ASCII (Topaz) isn't handled yet:
icy_engine picks the font from SAUCE, so a `.ASC` without SAUCE opens in
the IBM font.

Then editor, theme workshop, converters, animation, web app,
Windows/Linux (steps 3–8 in the notes).

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
  - `src/document.rs`: `Document`, a file opened for viewing: its info,
    the render settings (9-px, iCE, aspect) and `render_rows`, which
    renders a band of rows through icy_engine's renderer. Render modes
    never change the data, except that iCE moves the attribute's high bit
    between blink and bright background, which is reversible. Parsed
    formats end at their last row with content; binary formats keep their
    height.
  - `src/sauce.rs`: `SauceInfo`, every SAUCE field, decoded from CP437.
  - `examples/render_png.rs`, `examples/png_diff.rs`: for
    `scripts/compare-ansilove.sh`.
  - Later: converters, generators, the animation encoder, the contrast
    checker, the theme-pack model and `stylus-render`.
- `src-tauri/src/lib.rs`: Tauri commands wrapping the core, for file I/O
  and OS integration only.
- `src/lib/backend.ts`: the one frontend adapter over the core (Tauri now;
  WASM for the web and the editor's hot path later).
- `src/components/ArtViewer.tsx`: draws the art, one `<canvas>` per band
  of about 2048 px (one canvas can't hold a long ANSI), with a
  determinate progress bar over the rows. Aspect and zoom are CSS
  scaling. Blink alternates two frames once a second, never under reduced
  motion. `SaucePanel.tsx` beside it.
- `src/App.tsx`: the screen. `STARTUP_STEPS` feeds `StartupScreen`; add a
  step for each launch-time load (font atlases, recent files, the theme
  pack).
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

- **WASM check (the gate): icy_engine fails, checked 2026-09-29** at
  `b85e565` (`cargo check --target wasm32-unknown-unknown` in
  `stylus-core/`). What blocks it:
  - **icy_net** (tokio → mio, rustls → aws-lc-sys): the hard failure.
    icy_engine uses it only for one enum, `icy_net::telnet::TerminalEmulation`
    (3 references in `screen_modes.rs` and `formats/file_format.rs`).
  - **unarc-rs** (archives, incl. unrar): used in `error.rs` and
    `formats/file_format.rs`, including the public `FileFormat::Archive`
    variant.
  - **zstd-sys** (C code): icy_engine uses zstd directly (`.icy` files)
    and retrofont needs it through zip. It fails only because Apple's clang
    can't target wasm32; zstd-sys ships a wasm shim, so a wasm-capable
    clang (Homebrew `llvm`, via `CC_wasm32_unknown_unknown`) should build
    it. **Unverified:** on 2026-09-29 `brew install llvm` failed four
    times, because the ~390 MB bottle download from ghcr.io kept dropping
    (HTTP/2 `PROTOCOL_ERROR`, then `transfer closed` over HTTP/1.1). Retry
    on another network, then post the result on icy_tools#186, which
    promises a follow-up.
  - **Everything else in icy_engine's tree builds for wasm32**:
    icy_parser_core, icy_sauce, libyaff, codepages, icy_sixel, image,
    jxl-oxide, lodepng, gif, quantette, png, flate2, zip without zstd.
    Rayon compiles, but spawning threads panics on the web at run time.
  - **Options:** (a) propose `net` and `archives` feature flags upstream
    (small: move `TerminalEmulation` into icy_engine or icy_parser_core,
    and cfg the archive variant), plus wasm clang for zstd. This keeps
    all of icy_engine and drops unrar too. (b) Depend on
    `icy_parser_core` + `icy_sauce` only (they build for wasm32 today) and
    write our own buffer and binary-format loaders. Chose (a):
    **[icy_tools#186](https://github.com/mkrueger/icy_tools/pull/186)**,
    opened 2026-09-29 (`docs/upstream/icy_engine-net-archives-features.md`).
    Once merged: pin the new revision with `default-features = false,
    features = ["minimal"]`, update `ICY_TOOLS_REV`, rerun the license
    scan. Step 2 can start on native meanwhile, since the core's API
    doesn't change.
- **Art-file rendering fixes for icy_engine:**
  **[icy_tools#187](https://github.com/mkrueger/icy_tools/pull/187)**,
  opened 2026-09-29: BEL/DEL as • and ⌂ in art files, and iCE leaving 24-bit
  backgrounds alone. `docs/upstream/icy_engine-art-control-glyphs-ice-rgb.md`
  and `.patch`. Until it's merged, Stylus shows those 7 corpus files
  wrong. Once merged: pin the revision, then rerun
  `scripts/compare-ansilove.sh` and expect every file to match.
- **`unrar_sys`** (through unarc-rs) vendors RARLAB's UnRAR source under
  its freeware license: not copyleft, but not OSI open source. Decide
  whether an MIT app may ship it; the feature-flag route above would drop
  it.
- **icy crates without a license field**: `icy_engine` (repo MIT OR
  Apache-2.0), and `codepages`, `icy_net` (icy_board repo, Apache-2.0).
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
