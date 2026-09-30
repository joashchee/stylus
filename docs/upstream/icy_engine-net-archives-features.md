# Upstream proposal: `net` and `archives` features for icy_engine

**Status: closed unmerged, 2026-09-30.** The maintainer solved it at the
source instead, preferring Rust-only fixes over feature flags:
`TerminalEmulation` moved to `icy_terminal_emulation` (icy_board
`bf5d2fb`), pure-Rust `ruzstd` for `.icy` files (`899f8bec`), C-free
unarc-rs 0.7 and retrofont (`455869c3`), and icy_engine off icy_net
(`2e4e2b1b`). icy_engine builds for wasm32 with its default features;
Stylus pins `2e4e2b1`. Kept as a record; don't apply the patch.

Originally opened 2026-09-29 as
[mkrueger/icy_tools#186](https://github.com/mkrueger/icy_tools/pull/186),
from `joashchee/icy_tools` branch `wasm-feature-flags` (commit `4b1b7f0`,
on `b85e565`). The patch is `icy_engine-net-archives-features.patch` next
to this file. Everything below the line is the PR description as sent.

Why Stylus needs it: `stylus-core` has to build for
`wasm32-unknown-unknown` (the web app, and the editor's hot path in the
desktop webview), and it can't while icy_engine always pulls in icy_net's
networking stack. It also removes `unrar_sys` (RARLAB's freeware UnRAR
source) from Stylus's tree. See `CLAUDE.md`, "Open before step 2".

Once merged: pin the new revision in `stylus-core/Cargo.toml` with
`default-features = false, features = ["minimal"]`, update
`ICY_TOOLS_REV`, rerun `scripts/license-scan.py`, and drop `unrar_sys`
and the icy_board crates from its `REVIEWED` list if they're gone.

---

## icy_engine: optional `net` and `archives` features (for wasm32)

Hi Mike, thanks for icy_tools. I'm building Stylus, a free MIT ANSI art
editor on icy_engine, and it needs to run in the browser. icy_engine
itself is nearly wasm-ready: apart from two dependencies (and zstd's C
build, which only needs the right clang; see below), everything builds
for `wasm32-unknown-unknown`. This PR makes those two optional, **on by
default**, so nothing changes for icy_draw, icy_term, icy_view or the
other crates.

### What blocks wasm32 today

- **icy_net** brings tokio (`mio` has no wasm32 support) and rustls
  (`aws-lc-sys`, C code). icy_engine only uses it for one type,
  `icy_net::telnet::TerminalEmulation` (in `screen_modes.rs` and
  `formats/file_format.rs`).
- **unarc-rs** (archive opening) brings unrar and more C code. It's
  used in `error.rs` and `formats/file_format.rs`.

### What this changes

- `Cargo.toml`: `icy_net` and `unarc-rs` become optional, behind two new
  features, `net` and `archives`, both in `default`.
- New `src/terminal_emulation.rs`, re-exported as
  `icy_engine::TerminalEmulation`:
  - with `net`, it's `pub use icy_net::telnet::TerminalEmulation`, the
    same type as today, so it still works with icy_net's telnet code;
  - without `net`, icy_engine defines the same enum itself (same variants
    and derives).
  - `screen_modes.rs` and `file_format.rs` now import it from the crate
    root. The doc example uses `icy_engine::TerminalEmulation`.
- `FileFormat::Archive`, `archive_format_from_extension`, `as_archive`,
  `open_archive` and `EngineError::Archive` are `#[cfg(feature =
  "archives")]`. `is_archive()` stays and returns `false` without the
  feature. `from_extension` checks archive extensions only when the
  feature is on (otherwise it falls through to ANSI, as for any unknown
  extension).
- The archive extension list moved from `extensions()` into an
  `archive_extensions()` helper, so it needs one `cfg` instead of 22.
  The list itself is unchanged.

### Tested

- `icy_engine` with default features, `net` only, `archives` only, and
  neither: builds with no warnings.
- `cargo test -p icy_engine --lib`, with default features and with
  `--no-default-features --features minimal`: 81 passed, 0 failed, both
  ways. (Without `archives`, cargo notes that the workspace's
  `lzma-rust2` patch is unused, as expected.)
- Without both features, icy_engine's dependency tree has no tokio,
  rustls, russh, unrar or unarc-rs.
- `cargo check --target wasm32-unknown-unknown` with
  `--no-default-features --features minimal`: the only remaining failure
  is `zstd-sys`, which needs a wasm-capable clang (not Apple's). I
  haven't built it with one yet and will follow up here when I have.

### Not in this PR

- zstd stays a hard dependency (`.icy` chunk compression uses it, and
  retrofont needs it through zip). It builds for wasm32 with a
  wasm-capable clang (`CC_wasm32_unknown_unknown`).
- rayon compiles for wasm32 but can't spawn threads there without extra
  setup. That's a separate question, only if a wasm caller hits a
  parallel path.
- A cleaner long-term home for `TerminalEmulation` might be
  `icy_parser_core`, with icy_net re-exporting it. That touches both
  repos, so this PR keeps it to icy_tools.

Happy to adjust names or structure to whatever you prefer.
