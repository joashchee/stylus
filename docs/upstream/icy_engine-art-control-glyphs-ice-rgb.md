# Upstream proposal: BEL/DEL glyphs in art files, RGB backgrounds under iCE

**Status:** opened 2026-09-29 as
[mkrueger/icy_tools#187](https://github.com/mkrueger/icy_tools/pull/187),
from `joashchee/icy_tools` branch `art-file-fixes` (commit `fe633b3`, on
`b85e565`). The patch is `icy_engine-art-control-glyphs-ice-rgb.patch` next
to this file. Everything below the line is the PR description as sent.

Why Stylus needs it: step 2's comparison against libansilove
(`scripts/compare-ansilove.sh`) on six Sixteen Colors packs, 162 files,
found 7 files that render differently, all from these two bugs. With the
patch, all 162 match ansilove's pixels exactly. Independent of #186 (it
touches different code), so either can merge first.

---

## icy_engine: draw BEL and DEL in CP437 art files; keep RGB backgrounds under iCE

Hi Mike, a second small one from building Stylus on icy_engine (after
#186). I compared icy_engine's renders with ansilove's on 162 files from
six Sixteen Colors packs (1995–2024), pixel by pixel. 7 files differed,
all from two bugs in `parser_sink.rs`. With this patch every file
matches, apart from ansilove's own quirks (it keeps trailing blank rows,
and crops `FILE_ID.DIZ`).

### 1. 0x07 and 0x7F in art files

In an art file (`is_terminal_buffer == false`), 0x07 and 0x7F are CP437's
• and ⌂ glyphs, and artists use them. The ANSI parser emits them as
`Bell` and `Delete`, so the sink drops the bullet and deletes a cell for
the house, and the rest of the row shifts left. ansilove (which only
interprets LF, CR, TAB, SUB and ESC) and PabloDraw draw them as glyphs.

Now `Bell` and `Delete` print the glyph when the screen isn't a terminal
buffer and the buffer is CP437. Terminals behave as before, and so do
PETSCII and ATASCII files, where the parser's `Delete` is a real delete
(the first version of this patch broke `output::petscii::test_seq`,
which is how I found that).

Examples: `mimic84/us-sac06.ans` and `us-dikom.ans` (0x7F),
`blndr2024b/CHECS-FALKOR.ANS` (0x07).

### 2. iCE with RGB backgrounds

With iCE colors, `display_attribute` turns a blinking cell's
background into `background() + 8`. `background()` returns 0 for RGB, so
a blinking cell with a PabloDraw 24-bit background (`ESC[0;R;G;Bt`)
becomes palette color 8, dark gray. The shift now applies to palette
backgrounds only.

Example: `fire-41/DIP-FIRE.ANS` (`ESC[5;43m` then `ESC[0;255;255;87t`,
SAUCE iCE on) showed dark gray where it should be yellow.

### Tests

- Four new unit tests in `parser_sink.rs`: art files draw BEL and DEL,
  terminals still don't print BEL, iCE leaves RGB backgrounds alone, and
  iCE still brightens palette backgrounds.
- `cargo test -p icy_engine --lib`: 85 passed. `cargo test -p icy_engine --tests`:
  all pass, including `output::petscii::test_seq`.

Happy to change anything to fit how you'd rather handle it.
