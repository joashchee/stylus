# Image to ANSI converters

The Image to ANSI panel runs every open-source image-to-ANSI converter
we found whose license allows use in closed-source commercial software,
each ported to Rust in `stylus-core/src/convert/` so it runs locally
(and in the web build later). This records the search, what's in, what's
out and why, and how the ports were checked. Written 2026-09-30.

This doesn't replace the image-to-ANSI converter planned in Diskette's
`docs/stylus-notes.md` ("Image to ANSI", written for Stylus with OKLab
matching and a theme-safe mode). The ports are a comparison bench and a
reference for building it.

## Rules the ports follow

- **Identified by origin.** Each converter's `ConverterInfo` names the
  project, its repository, copyright line, license, the commit ported,
  the original's settings used (in its own terms), and what the port
  changes and why. The panel shows all of it; About lists every
  converter's license in full, plus the libraries the ports carry code
  from (`LIBRARY_NOTICES`).
- **Same width, same writer.** Every converter runs at 80 columns and
  its cells go through one `.ANS` writer (`grid.rs`), so any difference
  between two files is a difference between the converters. The writer
  emits no line breaks: every row fills the SAUCE width, and viewers
  wrap by themselves (a CR LF after a full row adds a blank line in
  icy_engine and ansilove).
- **Settings for classic ANSI art where the original has them** (16
  colors, the VGA palette, CP437 glyphs), otherwise its default mode.
  Each converter keeps its own color depth: 16 colors, xterm 256, or
  24-bit.
- **Faithful down to the arithmetic.** The resamplers the originals use
  are ported too (Pillow's fixed-point resize, Go's x/image/draw,
  nfnt/resize, imaging, Jimp's resizer, CImg's), along with tie-breaking
  order (CLImage's k-d tree), float precision (f32 where the original
  uses it) and quirks (termenv losing one on the round trip through hex,
  two nonstandard grays in some 256-color tables).
- **Changes only where needed.** An `.ANS` can't hold glyphs outside
  CP437, so TerminalImageViewer's glyph table is cut to the ones CP437
  has, and ansipx's quarter-block transparency edges are off. Nor can it
  hold control codes, so chromatic's character set leaves out CP437's
  control-code glyphs (a setting it has). An 81st column would wrap, so
  aimg's trailing reset space, asciimatics's end-of-row dot and ansir's
  one-past-the-end column are left out. Tools that never enlarge, or
  never resize, get scaled to 80 columns. Random choices use a
  fixed-seed SplitMix64. Where an original's terminal printer changes
  what shows (hiptext leaves some cells on the default background), the
  port keeps it.

## How they were checked

Each original was built or run in a scratch folder (never the repo) and
compared with its port cell by cell (glyph, foreground, background) on
three images: the mandrill test image, a photo, and a generated image with
transparency. PNGs only, since JPEG decoders differ. A small crop (60×30) was
added for converters that enlarge differently from how they shrink.

Go, C and C++ originals are compared as built for x86-64 (Go with
`GOARCH=amd64`, run under Rosetta) or with `-ffp-contract=off`. Built
for arm64 they fuse multiply-adds, which can tip a rounding: ANSI-art
differed in 2 cells, ascii-image-converter in 3 and hiptext in 1 on the
test images. Where a library's math differs from the system's in the last
bit, the port carries the library's: Go's own `math.Sin`
(`util::go_sin`), and OpenCV's soft-float `exp` for its Gaussian kernels.
Two ports carry a whole resizer: ImageMagick's `ResizeImage` for ransid
(`magick.rs`, its Lanczos and Mitchell filters as a Q16 HDRI build runs
them) and stb_image_resize2's path for img_to_txt (`stbir.rs`). ransid's
check ran the original R code through the magick package built against
Homebrew's ImageMagick; CRAN's own macOS and Windows builds of magick
bundle other ImageMagick versions, which can round differently.

| Converter | Origin | License | Checked against the original |
|---|---|---|---|
| KOAN.ansi | github.com/koan-shdw/koan-ansi | MIT | Exact (Python) |
| img2ansi (Wes Brown) | github.com/wbrown/img2ansi | BSD-3-Clause | Exact (Go) |
| libcaca img2txt | github.com/cacalabs/libcaca | WTFPL | Exact (its `dither.c`, C) |
| png2ansi (1Hyena) | github.com/1Hyena/png2ansi | MIT | Exact (C++, on pre-scaled input) |
| ansify | github.com/widberg/ansify | MIT | 99.8–100% (Rust; its k-d tree breaks exact ties differently) |
| ansir | github.com/themadcreator/ansir | Apache-2.0 | Exact (CoffeeScript, chroma-js 3.2.0) |
| CLImage | github.com/pnappa/CLImage | MIT | Exact (Python) |
| TerminalImageViewer | github.com/stefanhaustein/TerminalImageViewer | Apache-2.0 (or GPL-3.0) | Exact with the same CP437 glyph cut (C++) |
| viuer | github.com/atanunq/viuer | MIT | Exact (Rust) |
| catimg | github.com/posva/catimg | MIT | Exact (C) |
| terminal-image | github.com/sindresorhus/terminal-image | MIT | Exact (Node) |
| imgcat (Stephen Solka) | github.com/trashhalo/imgcat | MIT | Exact (Go) |
| imgcat (Eddie Antonio Santos) | github.com/eddieantonio/imgcat | ISC | Exact (C) |
| termpix | github.com/hopey-dishwasher/termpix | Apache-2.0 | Exact (Rust, with its lockfile's `image` 0.19) |
| image-to-ansi | github.com/dom111/image-to-ansi | MIT | Exact, given the same scaled pixels (the browser does its scaling) |
| aimg | github.com/stroborobo/aimg | ISC | Exact (Go) |
| img2ansi (John McCabe) | github.com/johnmccabe/img2ansi | MIT | Exact (Go, on pre-scaled input) |
| img2txt (hit9) | github.com/hit9/img2txt | BSD-3-Clause | Exact (Python) |
| img2ansi (Bryan Matsuo) | github.com/bmatsuo/img2ansi | MIT | Exact (Go) |
| hiptext | github.com/jart/hiptext | Apache-2.0 | Exact (C++: its image, scaling, color and printer code, built with stand-ins for glog and gflags) |
| ansize | github.com/jhchen/ansize | MIT | Exact colors (Go; its characters are random) |
| ascii-image-converter | github.com/TheZoraiz/ascii-image-converter | Apache-2.0 | Exact (Go) |
| ASCII Magic | github.com/LeandroBarone/python-ascii_magic | MIT | Exact (Python) |
| img2ansi (Jakob Westhoff) | github.com/jakobwesthoff/img2ansi | MIT | Exact (Rust) |
| ansipx (ANSIzalizer) | github.com/Zebbeni/ansipx | MIT | Exact (Go) |
| asciimatics | github.com/peterbrittain/asciimatics | Apache-2.0 | Exact (Python) |
| term-image | github.com/AnonymouX47/term-image | MIT | Exact (Python) |
| Rich Pixels | github.com/darrenburns/rich-pixels | MIT | Exact (Python) |
| png2ansi (Théo Matricon) | github.com/Theomat/png2ansi | Apache-2.0 | Exact (C, on pre-scaled input) |
| img2ansi (Lukas Beranek) | github.com/lloiser/img2ansi | MIT | Exact (Go) |
| tapciify | github.com/tapciify/tapciify | MIT | Exact (Rust) |
| image-to-ascii | github.com/IonicaBizau/image-to-ascii | MIT | Exact (its lwip resize built from lwip's CImg 1.6.6 in C++, its pixel packages in Node; lwip itself no longer builds) |
| chromatic | github.com/crypt0lith/chromatic | MIT | Exact (Python, OpenCV 5.0) |
| ANSI-art | github.com/EtoDemerzel0427/ANSI-art | Apache-2.0 | Exact (Go) |
| img_to_txt | github.com/danny-burrows/img_to_txt | MIT | Exact (C; its stb_image_resize2 built with NEON and scalar alike, plus 25 random sizes) |
| ransid | github.com/coolbutuseless/ransid | MIT | Exact (R, magick 2.9.1 on ImageMagick 7.1.2 Q16 HDRI; plus 8 random sizes) |

## Found, permissive, not ported yet

None: every permissive converter found is ported. Tools found later go
in the same way (see the end of this document).

## Left out

- **Copyleft:** chafa (LGPL-3.0), timg (GPL-2.0), jp2a (GPL-2.0),
  img2irc (GPL-3.0), aalib (LGPL), ImageToANSI by cejudo (GPL-3.0).
  viuer's 256-color mode is left out for the same reason: it uses
  ansi_colours, which is LGPL. **MPL-2.0** (artem, pixterm): file-level
  copyleft, so porting their code would put the ported files under MPL.
- **No license** (all rights reserved): rez2ans, shadeans,
  minikomi/ansipix, torrycrass/image-to-ansi, samupl/img2ansi, and
  several small repositories without one.
- **Not image converters:** ansaconv (converts ANSI art for Unicode
  terminals), icy_draw (inserts images as sixels), ansilove and the other
  ANSI-to-PNG renderers.
- **Graphics protocols only** (sixel, kitty, iTerm2): nothing an `.ANS`
  can hold.
- **Unicode-only output** (braille, sextants, quadrants): ansir's braille
  and sub-block modes, ascii-image-converter's braille, KOAN's extended
  set. Their CP437 modes are used where they have one.

## Where the search looked

GitHub repository search ("image to ansi", "img2ansi", "png to ansi",
"image ansi art", "ansi art converter", "image ansi cp437" and similar),
crates.io, npm, and the tools these led to. Tools found later can be
added the same way: a module with its `ConverterInfo`, its license text
in `licenses/`, a place in `converters()`, and a cell-by-cell check
against the original.
