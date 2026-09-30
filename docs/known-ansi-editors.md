# Known ANSI Editors — Comprehensive Historical Catalogue

**Research scope:** Released software that can create or edit ANSI/text-mode art, including editors whose principal purpose is ANSI/ASCII art, plus a smaller group of general character-art editors that can actually produce ANSI output. Animation is recorded separately rather than being required.

**Library sources consulted:** `dos_preservation_catalogue.md` and `Amiga_Preservation_Catalogue.md`. The Library material emphasizes preservation of original applications, exact versions, native files, fonts, runtime environments, and disk images rather than relying solely on converted files.

> **Important:** “ANSI editor” is used here in the historical BBS/text-mode-art sense. ANSI art is not simply ASCII: it commonly uses ANSI escape sequences, CP437 characters, color attributes, custom fonts, and related BBS text-art formats.

## Legend

- **✓** = native capability confirmed
- **—** = no significant capability found
- **?** = insufficient evidence to establish it
- **Modern** = currently usable/maintained
- **Legacy** = released but effectively historical
- **Preserved** = old binaries/source remain obtainable

---

# 1. Master catalogue

| # | Editor | Era | Original / current platform | ANSI art | Animation | Status | Notes |
|---|---|---|---|---:|---:|---|---|
| 1 | **TheDraw** | 1986–93 | MS-DOS | ✓ | **✓** | Legacy / preserved | Foundational ANSI editor; fonts, animation, transitions |
| 2 | **ACiDDraw** | 1990s | MS-DOS | ✓ | ? | Legacy / preserved | Major successor to TheDraw; large ANSI canvases |
| 3 | **PabloDraw** | 1994–present | DOS → Windows/macOS/Linux | ✓ | ? | Active / preserved lineage | ANSI/ASCII + RIPscrip + collaboration |
| 4 | **Empathy** | 1990s | MS-DOS | ✓ | ? | Legacy / preserved | ACiD editor; custom-font/XBIN support |
| 5 | **iCE Draw / iCEDraw** | 1990s | MS-DOS | ✓ | ? | Legacy / preserved | iCE scene; IDF/custom font/palette support |
| 6 | **TundraDraw** | 2000s | Windows/Linux/macOS | ✓ | ? | Legacy / preserved | Qt; network collaborative drawing; TND/24-bit text art |
| 7 | **TetraDraw** | ~2002 | Unix/Linux | ✓ | — | Legacy / still packaged | GPL; TCP/IP multi-user drawing |
| 8 | **MysticDraw** | 1990s–2000s | DOS → Windows/Linux/BSD | ✓ | ? | Legacy / preserved | ANSI/ASCII, Avatar, BIN, PCB, TheDraw fonts |
| 9 | **IcyDraw** | 2020s | Windows/Linux/macOS | ✓ | **✓** | Active | Broad legacy-format support plus animation |
| 10 | **Durdraw** | 2010s–present | Linux/macOS/Unix | ✓ | **✓** | Active | Terminal editor; frame animation; CP437; 16/256 color |
| 11 | **Moebius** | 2010s–present | Windows/macOS/Linux | ✓ | — | Active / preserved | Modern Blocktronics editor |
| 12 | **Moebius XBIN / GJ / Beyond lineage** | 2020s | Windows/macOS/Linux | ✓ | — | Active forks | Extended fonts/colors and improvements |
| 13 | **teXt0wnz** | 2020s | Web/PWA | ✓ | ? | Active | Offline-capable browser ANSI editor |
| 14 | **ANSIDRAW** | 2020s | Web | ✓ | — | Active | Modern browser ANSI editor; CP437/CGA |
| 15 | **SyncDraw** | 2000s–2010s | Win/Linux/BSD/OS X/Solaris | ✓ | ? | Legacy / preserved | Up to 1,000 lines; TheDraw fonts; many BBS formats |
| 16 | **ANSI Text Editor (ATE)** | 2010s | Windows | ✓ | ? | Beta / preserved | 3,500+ fonts; broad ANSI/BBS format support |
| 17 | **REXPaint** | 2010s–present | Windows/Wine | ✓ | — | Released / maintained | General ASCII-art editor with ANSI import/export |
| 18 | **DarkDraw** | 1994 | MS-DOS | ✓ | ? | Legacy / preserved | Dedicated ANSI editor; VGA; iCEcolor |
| 19 | **AMPro ANSINator** | 1990s | MS-DOS | ✓ | ? | Legacy / preserved | Graphics-mode drawing converted to ANSI |
| 20 | **ANSIDraw (DOS)** | 1980s/90s | MS-DOS | ✓ | ? | Legacy / preserved | Historical DOS editor; distinct from modern web ANSIDRAW |
| 21 | **ArtworkX** | 1990s | MS-DOS | ✓ | ? | Legacy / preserved | Historical ANSI editor |
| 22 | **CIADraw** | 1990s | MS-DOS | ✓ | ? | Legacy / preserved | Historical ANSI editor |
| 23 | **ITPDraw** | 1990s | MS-DOS | ✓ | ? | Legacy / preserved | Historical ANSI editor |
| 24 | **PaintPro** | 1990s | MS-DOS | ✓ | ? | Legacy / preserved | Historical ANSI editor |
| 25 | **ShmANSI** | 1990s | MS-DOS | ✓ | ? | Legacy / preserved | Historical ANSI editor |
| 26 | **Aewan** | 2000s | Linux/Unix | ✓* | ? | Legacy | ASCII/text-art editor; secondary category |
| 27 | **BlockArt** | 2000s+ | Linux terminal | ✓ | ? | Legacy | Small ANSI/text-mode editor |
| 28 | **DarkDraw II** | modern | Windows/WSL/Linux/macOS | ✓* | **✓** | Active / released | Terminal art and animation editor; distinct from DOS DarkDraw |

\* These are better treated as text-mode/character-art editors that can produce or work with ANSI rather than as traditional BBS ANSI-scene editors.

---

# 2. The classic DOS generation

## 2.1 TheDraw

TheDraw is the essential historical ANSI-editor entry.

Released in 1986 by Ian E. Davis/TheSoft, with final public version **4.63 in 1993**, it could create ANSI and ASCII artwork and included its famous `.TDF` font system and dedicated ANSI animation facilities.

Its animation model is historically important: traditional ANSI animation was often a stream of terminal output whose playback behavior could depend on simulated modem/baud-rate timing rather than modern frame timing.

### Classification

- ANSI editor
- ASCII editor
- font editor
- ANSI animation editor
- transition/effect system

### Preservation importance

TheDraw should be preserved as a first-class reference implementation for this project.

---

## 2.2 ACiDDraw

ACiDDraw became another major DOS ANSI editor.

Historical documentation describes ACiDDraw as supporting creation/manipulation of ANSI/ASCII images, multiple 1,000-line editing pages, 160-column mode, VGA viewing, text justification, and numerous export formats.

Version **1.25r** was released publicly in September 1999.

### Classification

- ANSI editor
- ASCII editor
- large-canvas editor

ACiDDraw demonstrates that ANSI artwork was not inherently restricted to 80×25 screens.

---

## 2.3 Empathy

Empathy was another ACiD-associated DOS editor.

It was designed around demoscene/BBS ANSI use, with VGA palette support and keyboard conventions familiar to TheDraw and ACiDDraw.

The surviving ANSI-editor catalogues identify Empathy as an ANSI editor with custom-font functionality useful for creating XBIN artwork.

### Classification

- ANSI editor
- custom-font editor
- XBIN-related workflow

---

## 2.4 iCE Draw / iCEDraw

iCE Draw represents the iCE/ICEcolor side of the ANSI scene.

It used the **IDF** format, including custom character sets and palettes.

IDF remains part of the surviving text-mode-art format ecosystem and is still supported by modern ANSI libraries.

### Classification

- ANSI editor
- custom character-set editor
- palette editor
- IDF/XBIN ecosystem

---

## 2.5 DarkDraw

**DarkDraw 1.00** was released in October 1994.

Its surviving documentation describes it as an ANSI editor with a 3,000-line capacity, requiring a 386 and VGA, with optional mouse support.

### Classification

- DOS ANSI editor
- VGA editor
- large-canvas editor

---

## 2.6 Other DOS editors

Historical ANSI-art preservation sources identify the following additional DOS-era editors:

- ArtworkX
- CIADraw
- ITPDraw
- PaintPro
- ShmANSI
- ANSIDraw
- AMPro ANSINator

These should not be collapsed into TheDraw or ACiDDraw simply because documentation is less abundant.

---

## 2.7 AMPro ANSINator

ANSINator is conceptually interesting because its workflow was closer to a graphical paint program:

> graphics-mode painting → conversion/quantization → ANSI

This differs from the more traditional:

> character-cell painting → ANSI

That makes it particularly relevant as a historical example of an image-to-ANSI conversion workflow.

---

# 3. PabloDraw

PabloDraw is one of the most important bridges between the DOS and modern eras.

The PabloDraw lineage began in **1994** and had DOS releases before evolving into later graphical/cross-platform versions.

The modern project describes PabloDraw as an ANSI/ASCII and RIPscrip vector graphics editor/viewer with multi-user capabilities, with Windows, Linux and macOS support.

### Preservation targets

Treat at least these as separate historical records:

1. PabloDraw DOS
2. PabloDraw Windows
3. PabloDraw modern/cross-platform lineage

Do not collapse all versions into one generic application record when building a preservation registry.

---

# 4. TundraDraw

TundraDraw moved ANSI editing into a cross-platform graphical application.

It was a C++/Qt ANSI drawing program for Linux, Mac and Windows, with network connectivity allowing artists to draw together over the Internet.

It used the **TND** format and eventually supported 24-bit color.

### Classification

- ANSI editor
- collaborative editor
- TND editor
- 24-bit text-mode editor

---

# 5. TetraDraw

TetraDraw 2.0.2 is a Unix-era ANSI editor.

It was designed specifically for Unix operating systems and allowed multiple artists to edit an image simultaneously over TCP/IP.

The project is particularly notable because it remains available through modern Linux packaging despite its historical origin.

### Classification

- Unix ANSI editor
- multi-user editor
- TCP/IP collaborative drawing

---

# 6. MysticDraw

MysticDraw was designed explicitly as a replacement for TheDraw.

Its surviving project documentation describes support for:

- ANSI
- ASCII
- Avatar
- Binary
- PCBoard
- TheDraw fonts

It had Windows, Linux and BSD versions.

### Classification

- ANSI editor
- ASCII editor
- TheDraw-compatible editor
- multi-format BBS editor

---

# 7. SyncDraw

SyncDraw deserves its own entry rather than being treated merely as MysticDraw.

Its historical documentation lists support for:

- Windows 95/98/ME/NT/2000/XP
- Mac OS X
- Linux
- OpenBSD
- NetBSD
- FreeBSD
- Solaris/SPARC

and:

- IBM low/high ASCII
- TheDraw fonts
- ANSI
- Avatar
- PCBoard
- ASCII
- Binary
- C
- Synchronet formats
- SAUCE
- up to 1,000 lines

The surviving site still provides historical binaries.

### Important lineage

SyncDraw began from cleanup/development of an old MysticDraw version.

Therefore the registry should represent:

```text
MysticDraw
    ↓
SyncDraw
```

as a software lineage.

---

# 8. ANSI Text Editor (ATE)

ATE is a Windows 32/64-bit ANSI editor from the 2010s.

It was explicitly designed as an ANSI editor comparable to:

- PabloDraw
- ACiDDraw
- TheDraw
- TundraDraw
- TetraDraw
- AnsiDraw

It has a particularly ambitious font system with more than 3,500 ANSI/ASCII fonts and supports TheDraw font import/export.

### Reported formats

- ANS
- ASC
- PCB
- WC2
- WC3
- AVT
- BIN
- TXT
- PNG
- XBIN
- TND
- ADF
- IDF

It was beta software rather than a fully mature commercial product, but belongs in the historical catalogue.

---

# 9. REXPaint

REXPaint is not a traditional BBS ANSI editor, but it is a legitimate ANSI-capable character-art editor.

It can import/export ANSI art and is designed around character/tile graphics.

### Important limitation

Animated ANSI files are not its primary target; its documentation explicitly indicates that animated `.ans` files are ignored.

Therefore:

```text
Static ANSI: yes
ANSI animation: no
```

It belongs in the secondary ANSI/text-mode-art category.

---

# 10. Moebius

Moebius is one of the principal modern ANSI editors.

It runs on:

- Windows
- macOS
- Linux

and provides a graphical ANSI/ASCII canvas, including its characteristic half-block brush.

It also supports collaborative editing through a server.

### Animation

No strong evidence was found that the original Moebius is a frame-animation editor in the TheDraw/Durdraw sense.

Classification:

> Modern static ANSI editor.

---

# 11. Moebius XBIN / GJ / Beyond lineage

The Moebius lineage includes:

```text
Moebius
   ↓
Moebius XBIN
   ↓
Moebius XBIN GJ! Edition
   ↓
Moebius XBIN Ultimate Edition
   ↓
Moebius Beyond
```

These should be represented as related but distinguishable software records when exact binary preservation matters.

Moebius Beyond remains an ANSI/ASCII editor for Windows, macOS and Linux.

---

# 12. IcyDraw

IcyDraw is one of the most important modern descendants of the classic editors.

Its feature set includes:

- CP437
- ANSI/ASCII
- Artworx ADF
- Avatar
- BIN
- XBIN
- PCBoard
- iCE
- TundraDraw
- Ctrl-A
- Renegade
- TheDraw fonts
- font editing
- multiple fonts
- layers
- transparency
- RGB
- SAUCE
- Sixel
- palette import
- plugins
- animation
- animated GIF export
- ANSI animation export

### Classification

> Full-featured modern ANSI editor + legacy-format editor + animation editor.

This makes IcyDraw particularly important to the Multiplatform ANSI Art & Animation Editor project.

---

# 13. Durdraw

Durdraw is the clearest modern counterpart to animation-oriented editors such as TheDraw.

It supports:

- Linux
- macOS
- other Unix-like systems
- ANSI
- ASCII
- Unicode
- CP437
- 16 colors
- 256 colors
- mouse drawing
- brushes
- frame-based animation
- controlled frame timing
- HTML
- mIRC colors
- animated GIF workflows

Its controlled frame timing explicitly addresses a limitation of traditional ANSI animation, where playback timing historically depended on terminal/baud-rate behavior.

### Classification

> Terminal ANSI editor + frame animation editor.

---

# 14. teXt0wnz

teXt0wnz is a modern browser/PWA editor.

It supports:

- ANSI
- ASCII
- NFO
- XBIN
- BIN
- DIZ
- UTF-8 text
- CP437
- iCE colors
- custom XBIN palettes
- SAUCE
- historical/custom fonts
- drawing
- fill
- shapes
- selection
- mirror mode
- offline operation
- browser-based collaboration

It can be installed as a PWA and operate offline.

### Important architectural characteristic

It is not merely an online editor:

> browser runtime + installable/offline PWA + local ANSI workflow

This is highly relevant to a multiplatform editor architecture.

---

# 15. Modern ANSIDRAW

The modern **ANSIDRAW** is a browser-based ANSI editor and is explicitly unrelated to the old DOS AnsiDraw.

It supports:

- CP437
- authentic CGA palette
- ANSI import/export
- PNG export
- drawing
- lines
- boxes
- flood fill
- text
- selection
- infinite-height canvas
- image → ANSI conversion

It does not appear to have native animation editing.

### Important naming distinction

```text
ANSIDraw (DOS)
     ≠
ANSIDRAW (modern web)
```

They should be separate registry records.

---

# 16. DarkDraw II

DarkDraw II is a modern terminal art and animation editor and should **not** be confused with the original 1994 DOS DarkDraw.

It supports:

- Unicode
- 256 colors
- frame-by-frame animation
- custom character/color palettes
- Linux
- macOS
- Windows through WSL

Registry:

```text
DarkDraw
DarkDraw II
```

as separate projects.

---

# 17. Secondary text-mode editors

## Aewan

Aewan is a Unix ASCII-art editor.

It is relevant to text-mode graphics but is less directly tied to the classic CP437/BBS ANSI workflow.

Recommended registry classification:

```text
category = textmode-art-editor
traditional-bbs-editor = no
```

## BlockArt

BlockArt is a smaller Linux-terminal ANSI/text-mode editor aimed at BBS-style artwork.

It belongs in the secondary category because its historical footprint is much smaller than TheDraw, ACiDDraw, PabloDraw, or IcyDraw.

---

# 18. Animation-specific subset

For an ANSI Art & Animation Editor project, the most important distinction is:

| Editor | ANSI editing | Frame animation | Modern controllable timing |
|---|---:|---:|---:|
| **TheDraw** | ✓ | **✓** | No — traditional ANSI timing |
| **ACiDDraw** | ✓ | ? | ? |
| **IcyDraw** | ✓ | **✓** | **✓** |
| **Durdraw** | ✓ | **✓** | **✓** |
| **DarkDraw II** | ✓ / text-mode | **✓** | **✓** |
| **teXt0wnz** | ✓ | ? | ? |
| REXPaint | ✓ | — | — |
| PabloDraw | ✓ | ? | ? |
| Moebius | ✓ | — | — |
| Moebius Beyond | ✓ | — | — |
| SyncDraw | ✓ | ? | ? |
| TundraDraw | ✓ | ? | ? |
| TetraDraw | ✓ | — | — |
| ATE | ✓ | ? | ? |
| MysticDraw | ✓ | ? | ? |

The strongest confirmed animation-oriented editors from this research are:

1. **TheDraw**
2. **IcyDraw**
3. **Durdraw**
4. **DarkDraw II**

TheDraw's animation model is historically different from the latter three: traditional ANSI animation is a sequence of terminal output rather than modern independent frames with explicit timing.

---

# 19. Platform coverage

## MS-DOS

```text
TheDraw
ACiDDraw
Empathy
iCE Draw
DarkDraw
ArtworkX
CIADraw
ITPDraw
PaintPro
ShmANSI
ANSIDraw
AMPro ANSINator
PabloDraw DOS
```

## Windows 9x/NT/XP era

```text
PabloDraw
TundraDraw
MysticDraw
SyncDraw
ATE
REXPaint
```

## Unix / Linux

```text
TetraDraw
MysticDraw
SyncDraw
TundraDraw
Durdraw
Moebius
IcyDraw
```

## Classic Mac / macOS

```text
PabloDraw
TundraDraw
SyncDraw
Moebius
IcyDraw
Durdraw
```

## Modern browser

```text
teXt0wnz
ANSIDRAW
```

## Cross-platform modern

```text
PabloDraw
Moebius
Moebius Beyond
IcyDraw
Durdraw
teXt0wnz
```

---

# 20. The ANSI/text-mode format ecosystem

A major finding is that "ANSI editor" is really a family of related formats rather than one format.

Important formats include:

```text
.ANS       ANSI escape-sequence art
.ASC       ASCII art
.BIN       raw text-mode video memory
.XB/.XBIN  XBin
.IDF       iCE Draw
.TND       TundraDraw
.ADF       Artworx
.AVT       Avatar
.PCB       PCBoard
.WC2/WC3   Wildcat
.Ctrl-A    Synchronet
.Renegade  Renegade BBS
.RIP       RIPscrip/vector graphics
```

Modern ANSI libraries such as `libansilove` retain support for several of these formats, demonstrating that the supposedly obsolete ecosystem remains technically coherent.

Therefore the broader project scope should preferably be:

> **ANSI / ASCII / XBIN / text-mode art editor and animation system**

rather than simply "ANSI editor."

---

# 21. Recommended preservation model

The Library's DOS and Amiga preservation research strongly supports an environment-first model.

For each important editor, preserve:

```text
ORIGINAL MEDIA / DOWNLOAD
        ↓
DISK IMAGE / ARCHIVE
        ↓
OPERATING SYSTEM
        ↓
APPLICATION + EXACT VERSION
        ↓
FONTS + LIBRARIES + SUPPORT FILES
        ↓
ORIGINAL ANSI / ARTWORK
        ↓
MODERN DERIVATIVE
```

For an ANSI editor specifically, preserve:

```text
application
application version
source/binary if available
custom fonts
palette files
configuration
sample artwork
native project format
exported ANSI
SAUCE metadata
documentation
runtime environment
```

Do not rely solely on a PNG or plain-text conversion.

---

# 22. Master preservation set

For the Multiplatform ANSI Art & Animation Editor project, the most valuable initial preservation/reference set is:

### Historical DOS

1. TheDraw
2. ACiDDraw
3. Empathy
4. iCE Draw
5. DarkDraw
6. ArtworkX
7. CIADraw
8. ITPDraw
9. PaintPro
10. ShmANSI
11. ANSIDraw
12. AMPro ANSINator
13. PabloDraw DOS

### Transitional / cross-platform

14. PabloDraw
15. TundraDraw
16. TetraDraw
17. MysticDraw
18. SyncDraw
19. ANSI Text Editor
20. REXPaint

### Modern graphical

21. Moebius
22. Moebius XBIN
23. Moebius XBIN GJ Edition
24. Moebius Beyond
25. IcyDraw

### Modern terminal / animation

26. Durdraw
27. DarkDraw II

### Modern web

28. teXt0wnz
29. ANSIDRAW

### Secondary text-mode editors

30. Aewan
31. BlockArt

---

# 23. Most important reference implementations

For reverse-engineering/design purposes, the most useful cross-section is:

```text
TheDraw
   ↓
ACiDDraw
   ↓
PabloDraw
   ↓
TundraDraw
   ↓
Moebius
   ↓
IcyDraw
   ↓
Durdraw
```

This sequence covers:

- classic DOS ANSI editing
- TheDraw fonts
- ANSI animation
- large canvases
- custom character sets
- alternate ANSI/BBS formats
- graphical GUI editing
- collaborative editing
- modern RGB text mode
- modern frame animation
- cross-platform operation

A second branch is:

```text
MysticDraw
    ↓
SyncDraw
```

which is important for understanding BBS-oriented cross-platform editing.

---

# 24. Important design implications

The historical editor ecosystem suggests that a modern editor should treat the following as separate layers:

```text
DOCUMENT
 ├── canvas dimensions
 ├── cells
 ├── character code
 ├── foreground color
 ├── background color
 ├── blink / intensity
 ├── font
 ├── palette
 ├── SAUCE metadata
 └── format-specific metadata

ANIMATION
 ├── frame list
 ├── per-frame duration
 ├── loop mode
 ├── transition/delta information
 └── playback/export mode

FONT
 ├── CP437 glyphs
 ├── custom glyphs
 ├── TheDraw-compatible font
 ├── iCE/IDF font
 └── XBIN font

EXPORT
 ├── ANSI
 ├── ASCII
 ├── BIN
 ├── XBIN
 ├── IDF
 ├── TND
 ├── PCB
 ├── ADF
 ├── AVT
 ├── RIP
 ├── PNG
 ├── GIF
 └── modern text/HTML
```

This architecture reflects the historical diversity much better than treating an ANSI file as simply a grid of characters.

---

# 25. Historical conclusions

The surviving editor ecosystem falls into several recognizable generations:

### Generation 1 — DOS ANSI scene

```text
TheDraw
ACiDDraw
Empathy
iCE Draw
DarkDraw
```

Characteristics:

- CP437
- 16-color VGA
- custom fonts
- 80/132/160-column modes
- ANSI escape sequences
- BBS distribution
- modem-speed animation

### Generation 2 — cross-platform ANSI

```text
PabloDraw
TundraDraw
TetraDraw
MysticDraw
SyncDraw
```

Characteristics:

- graphical desktop interfaces
- network collaboration
- larger canvases
- multiple BBS formats
- cross-platform operation

### Generation 3 — modern graphical ANSI

```text
Moebius
IcyDraw
REXPaint
```

Characteristics:

- modern GUI
- mouse support
- high-resolution displays
- RGB/extended color
- modern operating systems
- legacy-format compatibility

### Generation 4 — terminal/frame-animation

```text
Durdraw
DarkDraw II
```

Characteristics:

- terminal-native workflows
- modern color capabilities
- explicit frame animation
- controlled timing

### Generation 5 — browser/PWA

```text
teXt0wnz
ANSIDRAW
```

Characteristics:

- zero-install access
- browser runtime
- offline/PWA operation
- modern UI
- easy cross-platform distribution

---

# 26. Bottom line

The known ANSI-editor ecosystem is substantially larger than the handful of commonly remembered programs.

The historically significant core is:

```text
TheDraw
ACiDDraw
PabloDraw
Empathy
iCE Draw
TundraDraw
TetraDraw
MysticDraw
SyncDraw
IcyDraw
Durdraw
Moebius
Moebius Beyond
teXt0wnz
```

with numerous smaller DOS editors surrounding them.

For **ANSI + optional animation**, the four most important animation references are:

```text
TheDraw
IcyDraw
Durdraw
DarkDraw II
```

For **format breadth**, IcyDraw and PabloDraw are particularly useful reference points.

For **modern GUI architecture**, Moebius and IcyDraw are important.

For **terminal-native animation**, Durdraw is particularly relevant.

For **historical authenticity**, TheDraw, ACiDDraw, Empathy and iCE Draw are indispensable.

For **offline/browser architecture**, teXt0wnz is particularly relevant.

The complete preservation collection should retain not just executables, but exact versions, custom fonts, sample artwork, native formats, palettes, documentation, configuration, and the operating-system/runtime environment in which each editor originally operated.
