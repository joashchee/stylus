# Upstream fix: libansilove BinaryText black-with-blink background

**Status:** opened 2026-09-29 as [ansilove/libansilove#28](https://github.com/ansilove/libansilove/pull/28), from `joashchee/libansilove` branch `binary-blink-black` (commit `ead68a1`, on `e22973d`). The patch is `libansilove-binary-blink-black.patch` next to this file. Only affects `scripts/compare-ansilove.sh` (the known .BIN difference in its header); Stylus doesn't use libansilove.

---

## Fix black background with the blink bit in BinaryText files

In `src/loaders/binary.c`, the background's high bit is dropped when iCE colors are off:

```c
if (background > 8 && !options->icecolors)
	background -= 8;
```

Background 8 isn't included, so a cell with attribute 0x80–0x8F (black background, blink bit set) renders with a dark gray background (color 8) instead of black. Backgrounds 9–15 are already handled. The other loaders don't have this test.

**To reproduce:** a BinaryText file whose cells have attribute 0x80, rendered with `icecolors` off (e.g. `ansilove file.bin`), shows dark gray backgrounds. PabloDraw, and ansilove itself for attributes 0x90–0xFF, drop the high bit.

**Fix:** `>= 8`.

**Tested:** built libansilove (master, e22973d) before and after, and rendered a 160-column BIN mixing attributes 0x10–0x1F, 0x80–0x8F and 0x90–0x9F with `ansilove_binary`, `icecolors` off. Before, the 0x8X cells differ from a reference render (Stylus, an MIT editor on icy_engine); after, the images are pixel-identical. iCE rendering is unchanged, since the test only runs with `icecolors` off.

Found while comparing renders of Sixteen Colors packs pixel by pixel. Thanks for libansilove!
