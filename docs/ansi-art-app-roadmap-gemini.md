# ANSI Art & Animation Editor: Phased Implementation Roadmap

This document outlines a structured, four-phase feature roadmap for developing a comprehensive, multiplatform ANSI Art & Animation Editor. The features are derived from a historical analysis of text-mode editors spanning five generations—from classic DOS tools (TheDraw, ACiDDraw, iCE Draw) to cross-platform and modern editors (PabloDraw, TundraDraw, Moebius, IcyDraw, Durdraw, teXt0wnz, DarkDraw II).

---

## Architecture & Data Model (Core Foundation)

Before feature implementation, the editor architecture must decouple canvas data from export targets by supporting the following core model:

```text
DOCUMENT
 ├── Canvas Dimensions (80x25 baseline up to 1,000–3,500+ lines)
 ├── Cells
 │    ├── Character Code (CP437 / Unicode)
 │    ├── Foreground Color
 │    ├── Background Color
 │    ├── Blink / Intensity Flag
 │    └── Font Reference ID
 ├── Active Palette (16-color CGA/VGA, iCE, XBIN, 256-color, 24-bit RGB)
 ├── Custom Character Sets / Fonts
 ├── SAUCE Metadata
 └── Format-Specific Metadata

ANIMATION
 ├── Frame List
 ├── Per-Frame Duration & Timing
 ├── Loop Mode
 ├── Transition / Delta Information
 └── Playback Mode (Modem Baud Simulation vs. Explicit Frame-Rate)
```

---

## Phase 1: Core Features (Standard Baseline)
*Features present in every standard ANSI/ASCII character-cell editor.*

### 1. Canvas & Viewport Management
* **Standard Grid Dimensions:** Baseline support for standard 80x25 character grid canvas.
* **Extended Display Modes:** Support for standard extended column widths (132 and 160 columns).

### 2. Character Set & Color Basics
* **CP437 Support:** Full Code Page 437 (IBM PC character set) glyph support.
* **Cell Attributes:** Character cell metadata handling glyph, foreground color, background color, and blink attribute.
* **Classic 16-Color Palette:** Standard CGA/VGA 16-color palette (8 low-intensity dark colors, 8 high-intensity foreground colors).

### 3. Drawing Tools & Input
* **Freehand Character Brush:** Freehand drawing using standard character placement.
* **Single-Cell Edit:** Direct character placement, replacement, and cell selection.
* **Input Options:** Full keyboard cursor control and basic mouse coordinate drawing support.

### 4. Native I/O
* **Standard ANSI Export/Import:** Native saving and loading of plain ANSI escape sequence files (`.ANS`).
* **Plain ASCII Export/Import:** Plain ASCII text files (`.ASC` / `.TXT`).

---

## Phase 2: Major Reference Features (Industry-Standard Editors)
*Features found across mainstream historical and modern flagship reference editors such as **TheDraw**, **ACiDDraw**, **PabloDraw**, **Moebius**, and **IcyDraw**.*

### 1. Canvas Expansion & Block Tools
* **Large Canvases:** Support for multi-page, infinite-height, or long scrollable canvases (up to 1,000–3,500+ lines, as seen in ACiDDraw and DarkDraw).
* **Half-Block Drawing Brush:** Sub-cell block drawing tool (pioneered by Moebius for high-detail block art).
* **Geometric Shapes:** Precision line drawing, box/rectangle tools, and flood fill algorithms.

### 2. Advanced Cell Manipulation & Color Modes
* **Selection & Transforms:** Rectangular selection with copy, paste, move, horizontal/vertical mirror/flip, and regional clearing.
* **iCE Color Mode:** Toggleable iCE color support (allowing 16 distinct background colors instead of 8 background colors + background text blink).

### 3. Typography & Font Engines
* **TheDraw Font (.TDF) Engine:** Support for loading, rendering, and typing multi-line outline and block text using `.TDF` font files.
* **Multi-Font Support:** Ability to assign multiple active character sets within a single workspace.

### 4. Ecosystem Formats & Metadata
* **Broad BBS/Text-Mode File Formats:** Import and export capabilities for:
  * Raw Binary Screen Dumps (`.BIN`)
  * Extended BIN (`.XB` / `.XBIN`)
  * iCE Draw (`.IDF`)
  * Artworx (`.ADF`)
  * Avatar (`.AVT`)
* **SAUCE Metadata Integration:** Full reading, writing, and editing of SAUCE (Standard Architecture for Universal Comment Extensions) records (Title, Artist, Group, Date, Comments, Aspect Ratio settings).

### 5. Collaboration & Frame Animation Basics
* **Network Collaboration Engine:** Multi-user live collaborative canvas synchronization over TCP/IP sockets (pioneered by PabloDraw, TundraDraw, and TetraDraw).
* **Frame-by-Frame Animation System:** Frame sequencing, frame insertion/deletion, playback controls, and controllable frame-rate timing (reference implementations: Durdraw and IcyDraw).

---

## Phase 3: Comprehensive Historical Feature Set (Specialized Editors)
*Features implemented across niche, specialized, or modern generation text-mode tools (**Durdraw**, **DarkDraw II**, **teXt0wnz**, **ATE**, **AMPro ANSINator**).*

### 1. Extended Palette & Modern Color Systems
* **256-Color Palette Support:** xterm 256-color palette support for modern terminal rendering.
* **24-Bit True-Color Mode:** Full 24-bit RGB text art mode (as seen in TundraDraw and IcyDraw).
* **Custom Palette Editor:** Creation, modification, and import of custom color palettes (TND, iCE, and XBIN custom palettes).

### 2. Advanced Compositing & Layering
* **Layer System:** Multi-layer canvas compositing with ordering, visibility toggles, and opacity/transparency controls (pioneered in IcyDraw).
* **Cell Transparency Modes:** Per-cell transparent foreground/background values.

### 3. Expanded BBS & Legacy Ecosystem Formats
* **Deep BBS Format Support:** Native parsing and rendering for:
  * PCBoard (`.PCB`)
  * Wildcat (`.WC2` / `.WC3`)
  * Synchronet (`Ctrl-A`)
  * Renegade BBS (`.Renegade`)
  * TundraDraw (`.TND`)

### 4. Expanded Font & Asset Engines
* **Comprehensive Font Catalog:** Deep custom font engine supporting custom character glyph creation, custom font bitmap editing, and large font library search (inspired by ATE's 3,500+ font catalog).

### 5. Graphic Quantization & Modern Media Export
* **Image-to-ANSI Converter:** Automated bitmap-to-character-cell rasterizer with custom color quantization and palette matching (inspired by AMPro ANSINator and modern web ANSIDRAW).
* **Modern Media Export Targets:**
  * Animated GIF export for ANSI animations.
  * Sixel graphics rendering output.
  * HTML / mIRC color-coded output export.
  * High-resolution PNG raster rendering.

### 6. Flexible Operating Environments
* **Multi-Environment Execution:**
  * Web / Offline Progressive Web App (PWA) architecture (inspired by teXt0wnz).
  * Terminal-native CLI execution mode (inspired by Durdraw and DarkDraw II).
  * Graphical cross-platform desktop UI execution (macOS, Windows, Linux).

---

## Phase 4: Unimplemented, Speculative & Advanced Features
*Features mentioned in historical design notes, community feature requests, or wishlist specifications across the ecosystem, but never fully integrated into mainstream ANSI art software.*

### 1. Baud-Rate Playback Simulation Engine
* A hardware-accurate modem speed simulation playback system (300, 1200, 2400, 9600, 14.4k, 33.6k baud) for rendering ANSI art and animations based on exact character-per-second output rates to recreate authentic BBS dial-up display behavior.

### 2. Hybrid Vector-to-Character Engine (RIPscrip Hybridization)
* A unified canvas engine enabling live, non-destructive editing and conversion between RIPscrip vector graphics primitives (lines, circles, polygons) and character-cell ANSI layers on the same canvas.

### 3. Procedural Shader & Effect Matrices
* Procedural, non-destructive character cell transformations across frame timelines, such as automated rain/matrix glitches, wave distortions, interactive palette cycling, and per-cell displacement maps.

### 4. Audio-Synced Animation Timeline
* A multi-track animation editor featuring audio track alignment, enabling ANSI animation frames to sync precisely to custom tracker audio (`.MOD`, `.XM`, `.S3M`) or MIDI files.

### 5. Delta-Compressed ANSI Stream Exporter
* An optimization exporter that calculates minimal byte-difference ANSI escape sequences between consecutive frames, producing ultra-compact ANSI animation streams optimized for low-bandwidth streaming.

### 6. Non-Destructive Cell Blending Modes
* Layer blending algorithms tailored specifically to character-cell graphics (e.g., character overlay masks, color intensity burn/dodge, and non-destructive background glyph retention).