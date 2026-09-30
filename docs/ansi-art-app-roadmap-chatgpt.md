# ANSI Art App — Comprehensive Feature Roadmap

This roadmap treats the application as a **historical superset** of ANSI/ASCII/text-mode editors. The phases distinguish foundational/common capabilities, features demonstrated by major reference editors, the full historically implemented feature envelope, and ideas that were mentioned or technically envisioned but were not established capabilities in the surveyed ecosystem.

---

# Phase 1 — Core Features

**Goal:** A usable ANSI/ASCII editor covering the fundamental capabilities shared by essentially all serious editors.

## 1. Canvas
- [ ] Create a new canvas
- [ ] Configurable width
- [ ] Configurable height
- [ ] Fixed-height canvas
- [ ] Expandable/infinite-height canvas
- [ ] Vertical/horizontal scrolling
- [ ] Cursor positioning
- [ ] Home/end navigation
- [ ] Page navigation
- [ ] Jump to line
- [ ] Clear, crop, and resize canvas
- [ ] Preserve artwork outside visible viewport
- [ ] Configurable background character/color

### Historical defaults
- 80×25
- 80×50
- 132-column modes
- 160-column modes
- Arbitrary dimensions

## 2. Character Editing
- [ ] Place, replace, delete, and insert characters
- [ ] Overwrite mode
- [ ] Text insertion mode
- [ ] Character picker
- [ ] ASCII and CP437 entry
- [ ] Character-code entry
- [ ] Character search
- [ ] Repeat last character
- [ ] Repeat character pattern
- [ ] Tab handling
- [ ] Space handling

## 3. Color
- [ ] Foreground color
- [ ] Background color
- [ ] Standard ANSI 16-color palette
- [ ] Bright colors
- [ ] Color picker/swatches
- [ ] Attribute display
- [ ] Copy color from existing character
- [ ] Change color without changing character
- [ ] Change character without changing color
- [ ] Default attribute
- [ ] Per-cell attributes

### Core cell representation
```text
character
foreground
background
attributes
```

## 4. Basic Drawing Tools
- [ ] Pencil
- [ ] Eraser
- [ ] Line
- [ ] Rectangle
- [ ] Filled rectangle
- [ ] Box
- [ ] Fill
- [ ] Text tool
- [ ] Character stamping
- [ ] Repeated-character drawing
- [ ] Horizontal line
- [ ] Vertical line
- [ ] Freehand drawing

## 5. Selection
- [ ] Rectangular selection
- [ ] Select all
- [ ] Cut/copy/paste
- [ ] Delete selection
- [ ] Move selection
- [ ] Duplicate selection
- [ ] Clear selection
- [ ] Undo/redo

## 6. Text Input
- [ ] Normal keyboard typing
- [ ] CP437 text
- [ ] Paste text
- [ ] Preserve attributes when typing
- [ ] Choose current font/character set
- [ ] Text alignment
- [ ] Multi-line text

## 7. File Operations
- [ ] New/open/save/save as
- [ ] Recent files
- [ ] Import/export
- [ ] File type detection
- [ ] Unsaved-change detection
- [ ] Backup/recovery
- [ ] Autosave

## 8. Fundamental ANSI Formats
- [ ] ANSI `.ANS`
- [ ] ASCII `.ASC`
- [ ] Plain text
- [ ] CP437
- [ ] ANSI escape sequences
- [ ] CR/LF conventions
- [ ] SAUCE metadata

## 9. Font Basics
- [ ] Built-in CP437 font
- [ ] Character-set viewer
- [ ] Character selection
- [ ] Character-set preview
- [ ] Import/export custom fonts
- [ ] Font preview
- [ ] Font switching

## 10. Viewport
- [ ] Zoom
- [ ] Fit canvas
- [ ] Actual-size view
- [ ] Grid
- [ ] Cursor visibility
- [ ] Attribute visualization
- [ ] Character boundaries
- [ ] Status bar
- [ ] Current character/color/coordinates
- [ ] Canvas dimensions

## 11. Project Architecture
```text
Document
 ├── Canvas
 ├── Cells
 ├── Palette
 ├── Font
 ├── Metadata
 └── History
```

**Do not make ANSI escape sequences the application's internal document format.**

---

# Phase 2 — Major Reference Editor Features

**Goal:** Reach the feature level demonstrated by major reference editors including TheDraw, ACiDDraw, iCE Draw, PabloDraw, TundraDraw, MysticDraw, ATE, Moebius, IcyDraw, and Durdraw.

## 12. Advanced Drawing
- [ ] Circle
- [ ] Ellipse
- [ ] Triangle
- [ ] Polygon
- [ ] Filled polygon
- [ ] Custom shapes
- [ ] Shape library
- [ ] Box-drawing mode
- [ ] Line-character selection
- [ ] Unicode block drawing
- [ ] Half-block drawing
- [ ] Quarter-block drawing
- [ ] Braille drawing
- [ ] Character-pair drawing

## 13. Advanced Selection
- [ ] Flip horizontal/vertical
- [ ] Rotate 90°/180°
- [ ] Mirror
- [ ] Stretch
- [ ] Scale
- [ ] Shear
- [ ] Repeat
- [ ] Tile
- [ ] Stamp
- [ ] Transparent paste
- [ ] Opaque paste
- [ ] Masked paste

## 14. Advanced Color
- [ ] Color replace
- [ ] Foreground replacement
- [ ] Background replacement
- [ ] Swap foreground/background
- [ ] Color remapping
- [ ] Palette editing
- [ ] Palette presets
- [ ] Palette import/export
- [ ] Custom palettes
- [ ] 16-color palettes
- [ ] 256-color palettes
- [ ] RGB/truecolor
- [ ] Color quantization
- [ ] Dithering

## 15. Character Sets / Fonts
- [ ] TheDraw `.TDF`
- [ ] iCE `.IDF`
- [ ] XBIN fonts
- [ ] Custom CP437 fonts
- [ ] Character-set editor/browser
- [ ] Character replacement
- [ ] Font transformations
- [ ] Font copying/conversion
- [ ] Font libraries
- [ ] Multiple fonts per document
- [ ] Font-specific character maps

## 16. ANSI Art Construction
- [ ] ANSI-aware text insertion
- [ ] ANSI escape preview
- [ ] Escape-sequence inspector
- [ ] Raw ANSI editor
- [ ] Attribute inspector/copying/painting
- [ ] Character painting
- [ ] Combined character/attribute painting

## 17. Legacy Format Support
- [ ] `.ANS`
- [ ] `.ASC`
- [ ] `.BIN`
- [ ] `.XBIN` / `.XB`
- [ ] `.ADF`
- [ ] `.IDF`
- [ ] `.TND`
- [ ] `.AVT`
- [ ] `.PCB`
- [ ] `.WC2`
- [ ] `.WC3`
- [ ] Ctrl-A
- [ ] Renegade
- [ ] TheDraw formats
- [ ] RIPscrip
- [ ] Avatar
- [ ] Synchronet
- [ ] mIRC color formats

## 18. SAUCE
- [ ] Read/write/edit SAUCE
- [ ] Title
- [ ] Author
- [ ] Group
- [ ] Date
- [ ] File size
- [ ] Data type
- [ ] File type
- [ ] Width/height
- [ ] Font information
- [ ] Comments
- [ ] Record preservation
- [ ] SAUCE viewer

## 19. ANSI Animation — Timeline
- [ ] Multiple frames
- [ ] Add/delete/duplicate/reorder frames
- [ ] Frame thumbnails
- [ ] Frame duration
- [ ] Frame counter
- [ ] Timeline
- [ ] Play/pause/stop
- [ ] Loop
- [ ] Reverse playback
- [ ] Step forward/backward

### Animation editing
- [ ] Onion skin
- [ ] Ghost previous/next frame
- [ ] Copy/paste frame
- [ ] Copy region between frames
- [ ] Hold frame
- [ ] Blank frame
- [ ] Transition frame

## 20. Historical ANSI Timing
- [ ] Frame delay
- [ ] ANSI playback timing
- [ ] Character-output timing
- [ ] Terminal-speed simulation
- [ ] Baud-rate simulation
- [ ] 300 baud
- [ ] 1200 baud
- [ ] 2400 baud
- [ ] 9600 baud
- [ ] Custom playback speed
- [ ] Delay sequences
- [ ] ANSI animation preview

This reproduces historical ANSI animation rather than merely modern GIF-style animation.

## 21. Modern Animation
- [ ] Constant frame rate
- [ ] Millisecond frame durations
- [ ] Variable frame rate
- [ ] Frame interpolation
- [ ] Looping
- [ ] Ping-pong
- [ ] Playback speed
- [ ] GIF export
- [ ] PNG sequence export
- [ ] ANSI animation export
- [ ] Terminal playback export

## 22. Layers
- [ ] Multiple layers
- [ ] Add/delete/duplicate/rename/reorder
- [ ] Hide/lock
- [ ] Layer opacity
- [ ] Merge
- [ ] Flatten
- [ ] Layer groups
- [ ] Per-layer palette
- [ ] Per-layer animation

## 23. Transparency
- [ ] Transparent cells
- [ ] Transparent foreground/background
- [ ] Transparent selection
- [ ] Alpha-like compositing
- [ ] Layer transparency
- [ ] Transparency preview

## 24. Import / Conversion
- [ ] Image → ANSI
- [ ] Image → ASCII
- [ ] Image → CP437
- [ ] Image → block art
- [ ] Image → half-block
- [ ] Image → Unicode
- [ ] Image → palette-reduced ANSI
- [ ] Image → grayscale ASCII
- [ ] PNG/GIF/JPEG import
- [ ] Animated GIF import

## 25. Export
- [ ] ANSI
- [ ] ASCII
- [ ] XBIN
- [ ] BIN
- [ ] PNG
- [ ] GIF
- [ ] Animated GIF
- [ ] HTML
- [ ] Plain Unicode text
- [ ] Terminal output
- [ ] Sixel
- [ ] SVG-like representation where appropriate
- [ ] Frame sequence

## 26. Collaboration
- [ ] Shared canvas
- [ ] Network connection
- [ ] Remote cursor
- [ ] Remote user indicator
- [ ] Live drawing
- [ ] Synchronized edits
- [ ] Chat
- [ ] Session host/join
- [ ] Save collaborative session
- [ ] Conflict handling

---

# Phase 3 — Every Historically Implemented Feature

**Goal:** Build the historical superset: a general-purpose text-mode art workstation.

## 27. Terminal / Text-Mode Rendering
- [ ] CP437
- [ ] ASCII
- [ ] Unicode
- [ ] ANSI
- [ ] XBIN
- [ ] iCE colors
- [ ] 256-color terminal
- [ ] Truecolor terminal
- [ ] ANSI escape inspection/editing
- [ ] Terminal preview
- [ ] Terminal-size simulation
- [ ] Cursor simulation
- [ ] Screen-state simulation
- [ ] ANSI control-code visualization

## 28. Extended Color Systems
- [ ] CGA
- [ ] EGA
- [ ] VGA
- [ ] iCE
- [ ] XBIN
- [ ] 256-color
- [ ] RGB
- [ ] HSL
- [ ] HSV
- [ ] Palette cycling
- [ ] Palette animation
- [ ] Palette locking
- [ ] Palette extraction
- [ ] Palette optimization

## 29. Special Character Systems
- [ ] CP437
- [ ] IBM PC extended characters
- [ ] ANSI line-drawing characters
- [ ] Block characters
- [ ] Shading characters
- [ ] Custom glyphs
- [ ] Unicode glyphs
- [ ] Unicode combining characters
- [ ] Emoji-aware mode
- [ ] Half-blocks
- [ ] Braille
- [ ] Sextants
- [ ] Quadrants

## 30. Text-Mode Painting Modes
- [ ] Character mode
- [ ] Attribute mode
- [ ] Foreground-only
- [ ] Background-only
- [ ] Character + attribute
- [ ] Box mode
- [ ] Brush
- [ ] Stamp
- [ ] Fill
- [ ] Replace
- [ ] Overlay
- [ ] Transparent

## 31. Brushes
- [ ] Single-character brush
- [ ] Multi-character brush
- [ ] Custom brush
- [ ] Brush library
- [ ] Rotation
- [ ] Mirroring
- [ ] Scaling
- [ ] Stamping
- [ ] Animated brushes
- [ ] Pattern brushes

## 32. Pattern Systems
- [ ] Pattern fill
- [ ] Repeating patterns
- [ ] Character patterns
- [ ] Color patterns
- [ ] Dither patterns
- [ ] Bayer dithering
- [ ] Error-diffusion dithering
- [ ] Custom dithering matrices
- [ ] Pattern editor
- [ ] Pattern library

## 33. Text Effects
- [ ] Outline
- [ ] Shadow
- [ ] Double shadow
- [ ] Gradient
- [ ] Color gradient
- [ ] Rainbow text
- [ ] Wave text
- [ ] Scrolling text
- [ ] Marquee
- [ ] Centering
- [ ] Justification
- [ ] Letter/character spacing
- [ ] Text stretching
- [ ] Text compression
- [ ] Text mirroring

## 34. ANSI Effects
- [ ] Screen clear
- [ ] Cursor positioning
- [ ] Color changes
- [ ] Delays
- [ ] Erase sequences
- [ ] Cursor hide/show
- [ ] Cursor movement
- [ ] Scrolling
- [ ] Screen transitions
- [ ] ANSI playback inspection

## 35. Animation Effects
- [ ] Wipe
- [ ] Dissolve
- [ ] Scroll
- [ ] Slide
- [ ] Reveal
- [ ] Type-on
- [ ] Character-by-character reveal
- [ ] Color transition
- [ ] Palette transition
- [ ] Region transition
- [ ] Random reveal
- [ ] Directional reveal

## 36. Frame Optimization
- [ ] Full-frame storage
- [ ] Delta frames
- [ ] Dirty-region detection
- [ ] Run-length encoding
- [ ] ANSI sequence optimization
- [ ] Duplicate-frame elimination
- [ ] Static-region detection
- [ ] Playback-size optimization
- [ ] Export-size estimation

## 37. ASCII / ANSI Conversion
- [ ] ANSI → ASCII
- [ ] ASCII → ANSI
- [ ] ANSI → XBIN
- [ ] XBIN → ANSI
- [ ] BIN → ANSI
- [ ] Image → ANSI
- [ ] ANSI → PNG
- [ ] ANSI → GIF
- [ ] ANSI → HTML
- [ ] Unicode → CP437
- [ ] CP437 → Unicode

## 38. RIPscrip
- [ ] RIP import/export
- [ ] RIP command inspection
- [ ] RIP drawing primitives
- [ ] RIP palette
- [ ] RIP fonts
- [ ] RIP animation/playback
- [ ] RIP-to-ANSI conversion
- [ ] ANSI-to-RIP conversion where representable

## 39. Avatar
- [ ] Avatar import/export
- [ ] Avatar command parsing
- [ ] Avatar preview

## 40. PCBoard
- [ ] PCB import/export
- [ ] PCB color handling
- [ ] PCB control sequences
- [ ] PCB preview

## 41. Synchronet
- [ ] Synchronet import/export
- [ ] Synchronet-specific control codes
- [ ] Synchronet preview

## 42. Renegade / Ctrl-A
- [ ] Ctrl-A parsing/export
- [ ] Renegade color sequences
- [ ] Format conversion
- [ ] Preview

## 43. mIRC Art
- [ ] mIRC color codes
- [ ] mIRC text import/export
- [ ] mIRC palette
- [ ] Conversion to ANSI

## 44. Metadata / Preservation
- [ ] SAUCE
- [ ] File comments
- [ ] Creation/modification date
- [ ] Author
- [ ] Group
- [ ] Original filename
- [ ] Source application/version
- [ ] Source operating system
- [ ] Font provenance
- [ ] Palette provenance
- [ ] Conversion history
- [ ] Export history
- [ ] Checksums
- [ ] Preservation notes

## 45. Historical Runtime Reproduction
- [ ] DOS rendering mode
- [ ] VGA/CGA/EGA simulation
- [ ] 80×25 simulation
- [ ] 80×50 simulation
- [ ] 132-column simulation
- [ ] Terminal emulator
- [ ] DOS-style keyboard behavior
- [ ] DOS code-page behavior
- [ ] ANSI.SYS behavior
- [ ] Terminal baud-rate simulation

## 46. Old-School UI Modes
- [ ] TheDraw-style interface
- [ ] ACiDDraw-style interface
- [ ] iCE-style interface
- [ ] DOS full-screen interface
- [ ] Terminal interface
- [ ] Modern GUI interface
- [ ] Minimal keyboard-only interface

## 47. Keyboard System
- [ ] Full keyboard navigation
- [ ] Custom keybindings
- [ ] Multiple keybinding profiles
- [ ] DOS-style keys
- [ ] Vim-style keys
- [ ] Emacs-style keys
- [ ] Function-key actions
- [ ] Macro recording/playback
- [ ] Command palette
- [ ] Keyboard shortcut editor

## 48. Automation
- [ ] Scriptable editor
- [ ] Batch conversion
- [ ] Batch image-to-ANSI conversion
- [ ] Batch format conversion
- [ ] Batch metadata editing
- [ ] Command-line interface
- [ ] Headless renderer
- [ ] Export scripting
- [ ] Animation scripting

## 49. Browser / Offline Architecture
- [ ] PWA
- [ ] Offline operation
- [ ] Local file access
- [ ] Browser storage
- [ ] Installable web app
- [ ] No-server editing
- [ ] Offline font library
- [ ] Offline palette library
- [ ] Offline documentation
- [ ] Shareable documents
- [ ] Browser collaboration

## 50. Image/ANSI Algorithms
- [ ] Brightness analysis
- [ ] Contrast analysis
- [ ] Edge detection
- [ ] Character-density selection
- [ ] Automatic glyph selection
- [ ] Palette reduction
- [ ] Color-distance calculation
- [ ] Perceptual color matching
- [ ] Aspect-ratio compensation
- [ ] ANSI character aspect correction
- [ ] Preview before conversion
- [ ] Conversion parameter presets

## 51. Advanced Editing History
- [ ] Unlimited undo/redo
- [ ] Branching undo
- [ ] Transaction-based undo
- [ ] Per-frame history
- [ ] Per-layer history
- [ ] Named history checkpoints
- [ ] Revert to checkpoint
- [ ] History browser

## 52. Multi-Document Editing
- [ ] Tabs
- [ ] Multiple open documents
- [ ] Split view
- [ ] Side-by-side comparison
- [ ] Frame comparison
- [ ] Difference view
- [ ] Overlay comparison
- [ ] Document cloning
- [ ] Cross-document copy/paste

## 53. Search / Analysis
- [ ] Search characters
- [ ] Search strings
- [ ] Search ANSI sequences
- [ ] Search colors
- [ ] Search attributes
- [ ] Replace characters
- [ ] Replace colors
- [ ] Replace attributes
- [ ] Find empty regions
- [ ] Find non-CP437 characters
- [ ] Find unsupported sequences
- [ ] ANSI validity checker

## 54. Preservation Diagnostics
- [ ] Detect malformed ANSI
- [ ] Detect unsupported escape codes
- [ ] Detect unknown SAUCE
- [ ] Detect missing fonts
- [ ] Detect missing palette
- [ ] Detect incompatible code page
- [ ] Detect truncation
- [ ] Detect corrupted XBIN
- [ ] Detect unsupported animation commands
- [ ] Generate preservation report
- [ ] Generate conversion report

---

# Phase 4 — Historically Mentioned / Proposed but Not Established

These are **not things to describe as established historical features**. They belong in a research/design backlog because they were discussed, envisioned, implied, or technically conceivable but did not become established capabilities in the surveyed editor ecosystem.

## 55. Fully Non-Destructive ANSI Editing

```text
Original ANSI
      ↓
semantic document model
      ↓
editable layers
      ↓
reversible export
```

- [ ] Preserve original escape sequences
- [ ] Preserve original formatting decisions
- [ ] Edit semantic cells without destroying source
- [ ] Re-export while retaining untouched original sequences
- [ ] Source-level diff
- [ ] Rendered-level diff

## 56. Lossless Round-Trip Editing

> Import → edit → export without changing anything that wasn't deliberately changed.

- [ ] Byte-preserving untouched regions
- [ ] Escape-sequence preservation
- [ ] SAUCE preservation
- [ ] Unknown-field preservation
- [ ] Unknown-control-code preservation
- [ ] Original line-ending preservation
- [ ] Original encoding preservation

## 57. Universal Text-Art Intermediate Format

A canonical internal format capable of representing:

```text
ANSI
ASCII
XBIN
BIN
iCE
TDF
IDF
PCB
AVT
RIP
Avatar
Synchronet
Ctrl-A
Unicode
RGB
layers
animation
metadata
```

- [ ] Universal import
- [ ] Universal export
- [ ] Lossless capability tracking
- [ ] Format capability matrix
- [ ] Automatic downgrade warnings

## 58. Format Capability Analysis

Before exporting, detect incompatible features and explain the resulting loss.

- [ ] Capability comparison
- [ ] Loss report
- [ ] Automatic fallback
- [ ] User-selectable degradation
- [ ] Preview degraded result
- [ ] Export recommendation

## 59. Historical Accuracy Mode

Example target:

```text
IBM VGA
CP437
80×25
16 colors
ANSI.SYS
TheDraw 4.x
```

Potential targets:

- [ ] CGA
- [ ] EGA
- [ ] VGA
- [ ] DOS ANSI
- [ ] TheDraw
- [ ] iCE
- [ ] XBIN
- [ ] PCBoard
- [ ] RIP
- [ ] Terminal ANSI
- [ ] Modern Unicode terminal

## 60. Emulation Profiles

- [ ] DOS profile
- [ ] TheDraw profile
- [ ] iCE profile
- [ ] PCBoard profile
- [ ] Synchronet profile
- [ ] mIRC profile
- [ ] Linux terminal profile
- [ ] xterm profile
- [ ] Windows Terminal profile

Each profile defines:

```text
palette
font
character set
screen size
escape sequences
timing
limitations
```

## 61. ANSI Playback Laboratory

- [ ] Step through every ANSI command
- [ ] Show resulting screen
- [ ] Show raw escape sequence
- [ ] Show cursor position
- [ ] Show current attributes
- [ ] Show timing
- [ ] Show simulated terminal state
- [ ] Pause at arbitrary byte
- [ ] Modify a sequence and replay
- [ ] Compare terminal implementations

This turns the editor into an **ANSI research tool**, not just an art application.

## 62. Automatic Historical Reconstruction

Given an old file, attempt to determine:

- [ ] Likely editor
- [ ] Likely format
- [ ] Likely font
- [ ] Likely palette
- [ ] Likely terminal
- [ ] Likely screen dimensions
- [ ] Likely era
- [ ] SAUCE information
- [ ] Animation behavior

Example report:

```text
Historical reconstruction:
  Format: ANSI
  Encoding: CP437
  Screen: 80×25
  Palette: VGA
  Likely environment: DOS
  Likely editor family: ...
```

Present **evidence and confidence**, rather than treating uncertain identification as fact.

## 63. AI-Assisted ANSI Conversion

- [ ] Image → ANSI
- [ ] Image → ASCII
- [ ] Text → ANSI art
- [ ] Sketch → ANSI
- [ ] Describe → ANSI
- [ ] Automatic palette selection
- [ ] Automatic character selection
- [ ] Automatic composition
- [ ] Automatic animation
- [ ] Style matching
- [ ] Historical-editor style matching

AI-generated output should be clearly marked as generated rather than historically authentic.

## 64. AI-Assisted Restoration

- [ ] Detect corrupted regions
- [ ] Detect missing characters
- [ ] Infer likely glyph
- [ ] Infer likely color
- [ ] Repair broken ANSI sequences
- [ ] Reconstruct missing frame
- [ ] Compare multiple copies
- [ ] Preserve original alongside restoration
- [ ] Record every restoration operation

## 65. Collaborative Historical Reconstruction

```text
Original artwork
       ↓
Research workspace
       ↓
Multiple restoration hypotheses
       ↓
Evidence comparison
       ↓
Final reconstruction
```

- [ ] Multiple hypotheses
- [ ] Annotated regions
- [ ] Sources per region
- [ ] Confidence values
- [ ] Reviewer comments
- [ ] Version history

## 66. Artwork Provenance Graph

```text
Original file
   ↓
Imported
   ↓
Converted
   ↓
Edited
   ↓
Restored
   ↓
Exported
```

- [ ] Parent document
- [ ] Source file hash
- [ ] Conversion application
- [ ] Application version
- [ ] Date
- [ ] Operator
- [ ] Font
- [ ] Palette
- [ ] Format
- [ ] Export format

## 67. Exact Historical Reproduction

> **"Reproduce this artwork exactly as it appeared in 1994."**

- [ ] Original font
- [ ] Original palette
- [ ] Original screen dimensions
- [ ] Original terminal behavior
- [ ] Original timing
- [ ] Original ANSI parser
- [ ] Original application version
- [ ] Original operating system profile
- [ ] Original keyboard behavior
- [ ] Original animation timing

## 68. Historical Editor Compatibility Test Suite

For each file:

```text
Original
↓
Import
↓
Render
↓
Export
↓
Re-import
↓
Pixel/cell comparison
```

Tests:

- [ ] Character equality
- [ ] Attribute equality
- [ ] Palette equality
- [ ] Dimension equality
- [ ] Metadata equality
- [ ] Animation equality
- [ ] Escape-sequence equivalence
- [ ] Visual equivalence

## 69. Format Fuzzing / Compatibility Testing

- [ ] Generate malformed ANSI
- [ ] Test parser
- [ ] Unknown escape codes
- [ ] Truncated files
- [ ] Invalid SAUCE
- [ ] Corrupt XBIN
- [ ] Corrupt font
- [ ] Oversized canvas
- [ ] Extreme colors
- [ ] Extreme frame counts

Useful for both preservation and security.

## 70. "Everything Mode"

Expose a master feature/format matrix.

| Capability | ANSI | XBIN | iCE | RIP | PCB | Unicode | Modern |
|---|---:|---:|---:|---:|---:|---:|---:|
| 16 colors | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| 256 colors | — | — | — | — | — | ✓ | ✓ |
| RGB | — | — | — | — | — | ✓ | ✓ |
| Custom font | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Layers | — | — | — | — | — | — | ✓ |
| Animation | ✓ | limited | limited | ✓ | limited | ✓ | ✓ |
| Transparency | — | — | — | — | — | limited | ✓ |
| SAUCE | ✓ | ✓ | ✓ | — | ✓ | — | ✓ |
| Terminal playback | ✓ | ✓ | — | — | — | ✓ | ✓ |

The actual matrix should eventually be generated from machine-readable format specifications rather than maintained manually.

---

# Recommended Development Order

The conceptual phases above are best implemented in this engineering sequence.

## Phase 1A — Engine

```text
Cell
Canvas
Palette
Font
Document
Undo
Serialization
```

## Phase 1B — Basic Editor

```text
Pencil
Line
Box
Fill
Text
Selection
Copy/Paste
Color
Zoom
```

## Phase 1C — ANSI

```text
CP437
ANSI parser
ANSI renderer
ANSI exporter
SAUCE
```

## Phase 2A — Historical Compatibility

```text
XBIN
BIN
iCE
TDF
IDF
PCB
AVT
TND
ADF
```

## Phase 2B — Advanced Art

```text
layers
transparency
brushes
patterns
advanced selection
palette tools
image import
```

## Phase 2C — Animation

```text
frames
timeline
timing
onion skin
ANSI playback
GIF export
```

## Phase 3A — Full Format Laboratory

```text
RIP
Avatar
Synchronet
Renegade
Ctrl-A
mIRC
legacy DOS formats
```

## Phase 3B — Modern Text Art

```text
Unicode
256 colors
RGB
half-block
braille
Sixel
modern terminals
```

## Phase 3C — Collaboration

```text
network canvas
remote cursors
live editing
session recording
```

## Phase 4 — Preservation / Research

```text
lossless round-trip
historical emulation
provenance
restoration
format diagnostics
compatibility testing
historical reconstruction
```

---

# Core Architecture

The most important conclusion from the historical survey is that **ANSI itself should not be the core abstraction**.

Use something closer to:

```text
                    ┌─────────────────────┐
                    │      Project        │
                    └──────────┬──────────┘
                               │
             ┌─────────────────┼─────────────────┐
             │                 │                 │
          Document          Metadata         Provenance
             │
       ┌─────┴─────┐
       │           │
    Canvas      Animation
       │           │
    ┌──┴──┐     ┌──┴──┐
    │     │     │     │
  Layers Palette Frames Timing
    │
   Cells
    │
 ┌──┼───────────────┐
 │  │       │       │
Char FG      BG   Attributes
 │
Font / Charset
```

Format adapters should surround the internal model:

```text
                    INTERNAL MODEL
                          │
       ┌──────────────────┼──────────────────┐
       │                  │                  │
     ANSI               XBIN               iCE
       │                  │                  │
      PCB                RIP              Avatar
       │                  │                  │
    TND/ADF             BIN             Unicode
       │                  │                  │
       └──────────────────┼──────────────────┘
                          │
                     EXPORTERS
```

This architecture gives the project something historically unusual:

> **One editor capable of representing the combined feature set of decades of ANSI/text-mode editors, while still being able to deliberately restrict itself to the capabilities of a particular historical machine, format, or editor.**

---

# The Four Phases in One Sentence

**Phase 1:** Make ANSI art properly.

**Phase 2:** Make the major historical editors' capabilities available.

**Phase 3:** Make the editor a superset of the entire ANSI/text-mode art ecosystem.

**Phase 4:** Make it a preservation, emulation, reconstruction, and research platform for capabilities the historical ecosystem never fully realized.

**Key distinction:** Phase 3 is the historical feature-completeness target; Phase 4 is where the project can genuinely go beyond the historical editors rather than merely cloning them.
