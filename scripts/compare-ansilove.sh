#!/bin/sh
# Renders every art file in a corpus folder with both Stylus (stylus-core's
# render_png example) and ansilove (libansilove, BSD-2-Clause), and compares
# the PNGs pixel by pixel. Build step 2's check (Diskette's
# docs/stylus-notes.md, "Test corpus").
#
# Usage: scripts/compare-ansilove.sh <corpus-folder> [report-folder]
#
# The corpus (e.g. a local clone of the Sixteen Colors archive) stays
# OUTSIDE this repo, and so does the report: that art has no license to
# redistribute (CLAUDE.md rule 9). The report folder defaults to
# $TMPDIR/stylus-ansilove. Needs `ansilove` on PATH (brew install ansilove).
#
# Both renderers use each file's SAUCE (ansilove -S), 8-px cells by
# default. Aspect correction is left out on both sides: it's a display
# stretch in Stylus. Writes report.txt: one line per file, "same" or what
# differs, then a tally.
#
# Known ansilove difference (libansilove 1.4.2, src/loaders/binary.c:112):
# in .BIN files without iCE, a black background with the blink bit
# (attributes 0x80-0x8F) comes out dark gray, because the test is
# `background > 8` where `>= 8` was meant. Stylus draws it black, which is
# right. A .BIN mismatch in only those cells is this, not a Stylus bug.
# Fix proposed: https://github.com/ansilove/libansilove/pull/28
#
# icy_engine keeps BEL (0x07) as a control code in art files, by its
# maintainer's choice (icy_tools#187): the ANSI format is terminal output,
# and control-code glyphs belong in .XB. ansilove draws it as •, so a file
# with BEL bytes differs in those cells. That's expected.
#
# Intended differences, not bugs (the report shows "0 of N shared pixels
# differ" with different sizes):
# - ansilove keeps trailing blank rows, where the cursor ended up. Stylus
#   stops at the last row with content.
# - ansilove crops FILE_ID.DIZ to its widest line. Stylus shows it at its
#   full width.
# - ansilove draws a SAUCE font of "IBM EGA" (8x14) in its 8x16 font, so
#   the same rows come out taller (blndr2020/c-mfs - blender.ans: 88 rows,
#   1408 px from ansilove, 1232 px from Stylus). Stylus uses the 8x14 font
#   the file names.
set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
CORPUS=${1:?usage: scripts/compare-ansilove.sh <corpus-folder> [report-folder]}
REPORT=${2:-${TMPDIR:-/tmp}/stylus-ansilove}

case "$(cd "$CORPUS" && pwd)/" in
  "$ROOT"/*) echo "The corpus must be outside the repo (CLAUDE.md rule 9)." >&2; exit 1 ;;
esac
command -v ansilove >/dev/null || { echo "ansilove isn't installed (brew install ansilove)." >&2; exit 1; }

cargo build --quiet --release --examples --manifest-path "$ROOT/stylus-core/Cargo.toml"
BIN="$ROOT/stylus-core/target/release/examples"

mkdir -p "$REPORT/stylus" "$REPORT/ansilove"
: > "$REPORT/report.txt"

# The formats both renderers read. Add extensions as they are checked.
find "$CORPUS" -type f \( -iname '*.ans' -o -iname '*.asc' -o -iname '*.nfo' -o -iname '*.diz' \
  -o -iname '*.bin' -o -iname '*.xb' -o -iname '*.adf' -o -iname '*.idf' -o -iname '*.tnd' -o -iname '*.pcb' \) | sort |
while IFS= read -r file; do
  key=$(printf '%s' "${file#"$CORPUS"/}" | tr '/ ' '__')
  ours="$REPORT/stylus/$key.png"
  theirs="$REPORT/ansilove/$key.png"
  if ! "$BIN/render_png" "$file" "$ours" 2>>"$REPORT/errors.txt"; then
    echo "stylus failed: $file" >> "$REPORT/report.txt"; continue
  fi
  if ! ansilove -S -q -o "$theirs" "$file" >/dev/null 2>>"$REPORT/errors.txt"; then
    echo "ansilove failed: $file" >> "$REPORT/report.txt"; continue
  fi
  printf '%s: %s\n' "$file" "$("$BIN/png_diff" "$ours" "$theirs" || true)" >> "$REPORT/report.txt"
done

total=$(grep -c '' "$REPORT/report.txt" || true)
same=$(grep -c ': same$' "$REPORT/report.txt" || true)
failed=$(grep -c ' failed: ' "$REPORT/report.txt" || true)
differ=$((total - same - failed))
printf '\n%s files: %s same, %s differ, %s failed to render\n' "$total" "$same" "$differ" "$failed" | tee -a "$REPORT/report.txt"
echo "Report: $REPORT/report.txt"
