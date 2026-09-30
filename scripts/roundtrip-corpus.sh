#!/bin/sh
# Saving tested on real art (docs/roadmap.md, Phase 1c): opens every art
# file in a corpus folder, saves it in its own format (when Stylus saves
# that) and as .ANS and .XB, reopens each save and compares the rendered
# pixels with the original's (stylus-core's roundtrip example).
#
# Usage: scripts/roundtrip-corpus.sh <corpus-folder> [report-file]
#
# The corpus stays OUTSIDE this repo, and so does the report (CLAUDE.md
# rule 9). The report defaults to $TMPDIR/stylus-roundtrip.txt: one line
# per file, then a tally. "Differ with a stated loss" is expected (24-bit
# colors into ANSI, a custom font into ANSI); "differ unexplained" and
# "failed" are bugs.
#
# Expected, not bugs:
# - A binary file (XB, BIN…) with blank rows at the bottom saved as ANSI
#   reopens shorter: ANSI ends at its last row with content.
set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
CORPUS=${1:?usage: scripts/roundtrip-corpus.sh <corpus-folder> [report-file]}
REPORT=${2:-${TMPDIR:-/tmp}/stylus-roundtrip.txt}

case "$(cd "$CORPUS" && pwd)/" in
  "$ROOT"/*) echo "The corpus must be outside the repo (CLAUDE.md rule 9)." >&2; exit 1 ;;
esac

cargo build --quiet --release --example roundtrip --manifest-path "$ROOT/stylus-core/Cargo.toml"
"$ROOT/stylus-core/target/release/examples/roundtrip" "$CORPUS" > "$REPORT"
tail -n 1 "$REPORT"
echo "Report: $REPORT"
