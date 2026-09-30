# Upstream fix: icy_engine TundraDraw writer's starting color

**Status: fixed upstream as `da0d287` (2026-09-30), and Stylus pins
it.** The maintainer landed the fix himself ("Start the TundraDraw writer
from the loader's default attribute (#188)") and closed the PR. Stylus's
TundraDraw test now runs, and `.tnd` is back in the every-format round
trip. The rest of this file is the original write-up, kept as a record.

Opened 2026-09-30 as
[mkrueger/icy_tools#188](https://github.com/mkrueger/icy_tools/pull/188),
from `joashchee/icy_tools` branch `tundra-initial-color` (commit
`1e46f46`, on upstream `2e5b2cf`, where the bug is still present). The
patch, with a regression test in `tests/format/tundra.rs` that fails
before the fix, is `icy_engine-tundra-initial-color.patch` next to this
file. Found at `2e4e2b1`.
Stylus's test `edit::tests::tundra_draw_keeps_black_text_at_the_start`
is `#[ignore]`d until the pinned revision has a fix, and
`every_save_format_reopens_as_the_same_art` skips `.tnd` until then.

---

## TundraDraw: black text before the first color change comes back light gray

`save_tundra` (`crates/icy_engine/src/formats/io/tundra.rs`) starts from

```rust
let mut attr = TextAttribute::from_u8(0, buf.ice_mode);
```

(black on black) and only writes a foreground color when a cell's color
differs from that. `load_tundra` starts from `TextAttribute::default()`
(light gray on black). So any cells with a black foreground before the
first color change are written with no color command and read back as
light gray.

**To reproduce:** an 80×1 buffer whose first cell is `A`, foreground 0,
background 1. Save with `FileFormat::TundraDraw.to_bytes`, load the bytes
again: the cell's foreground is `Palette(7)`, not black (the background
comes back right, since both sides start from black there).

**Fix:** start the writer from the same attribute the reader does,
`TextAttribute::default()`. The writer then emits a foreground command
for the black cell.

**Tested:** the new test fails before the change and passes after; all
`icy_engine` tests pass at `2e5b2cf`, and `cargo fmt --check` is clean.
With the patch applied to `2e4e2b1`, a document with every
foreground on every background (16×8 cells, iCE on and off) saves and
reloads with identical rendered pixels, and the one-cell case above
reads back black.

Found by Stylus's save round-trip tests. Thanks for icy_tools!
