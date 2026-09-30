# ANSI Editor UI Design — Comprehensive Cross-Phase Proposal

## Purpose

Design one UI that can accommodate the entire ANSI/ASCII/text-mode editor roadmap:

1. Phase 1 — core editor
2. Phase 2 — major reference-editor capabilities
3. Phase 3 — complete historical feature envelope
4. Phase 4 — preservation, emulation, reconstruction, diagnostics, and experimental features

The central principle is:

> **Keep everyday editing simple; expose advanced capabilities through progressive disclosure rather than permanently displaying every historical feature.**

---

# 1. What the Major References Teach Us

## TheDraw — keyboard-first canvas

TheDraw establishes the fundamental full-screen, character-oriented editing model:

- canvas-first workflow
- CP437 characters and colors
- block selection
- fill
- copy/move/paste
- mouse-assisted selection
- font management
- automatic line/corner character selection through Draw Mode
- ANSI animation
- transition animations
- extensive legacy formats
- comprehensive help

Its biggest UI lesson is that the **canvas remains dominant** and common operations need fast keyboard access. TheDraw also supported selected-area load/save, TDF font management, PCBoard/Wildcat/AVATAR and other formats, and built-in animation/transition tools. [TheDraw](https://en.wikipedia.org/wiki/TheDraw)

## ACiDDraw — large ANSI workspace

Historical ACiDDraw descriptions emphasize:

- four editing pages
- up to 1,000 lines per page
- 160-column editing
- VGA viewing
- text justification
- multiple save formats

UI implication: the canvas/navigation system must not be hard-coded to 80×25.

## iCE Draw — glyph/font-centric ANSI work

iCE-oriented editing demonstrates the importance of:

- custom character sets
- iCE colors
- font-centric workflows
- precise text-mode construction

UI implication: the active charset and palette must be first-class controls.

## PabloDraw — desktop GUI + collaboration

PabloDraw combines ANSI/ASCII editing and viewing with RIPscrip and multi-user drawing. Its current project describes it as an ANSI/ASCII text and RIPscrip vector-art editor/viewer with multi-user capabilities. [PabloDraw](https://github.com/cwensley/pablodraw)

UI implication: collaboration and specialized formats should be **workspaces/panels**, not permanent toolbar clutter.

## ATE — mature specialist ecosystem

ATE documents a particularly broad feature set:

- custom ANSI font system
- TheDraw font import/export
- vertical/horizontal flips
- flat and gradient fills
- recoloring
- color masks
- free paint
- grid
- unusual canvas sizes
- tall ANSI screens
- palette editor
- customizable function keys
- custom font editor
- glyph editor
- SAUCE metadata
- art-pack browser
- ANSI viewer

UI implication: fonts, glyphs, palettes, metadata and browsing should be **detachable specialist panels/workspaces**. [ATE](https://www.roysac.com/roy-tools/ansi-text-editor.html)

## Moebius — painter + text editor

Moebius's defining half-block brush deliberately makes ANSI editing feel closer to a graphical painting application while retaining text-editor/function-key interaction. [Moebius](https://github.com/blocktronics/moebius)

UI implication: support both a **cell-precise editing model** and a **brush/painter model** without changing documents.

## Moebius XBIN — modern specialist tools

The Moebius XBIN lineage adds useful modern concepts including:

- palette browser
- live palette preview
- favorites
- grid
- guides
- zoom
- reference images
- image opacity/transform controls
- canvas row/column insertion/deletion
- character-set remapping
- glyph picking

UI implication: modern specialist controls should be contextual and dockable rather than always visible.

## REXPaint — modern desktop editing

REXPaint demonstrates:

- separate character/foreground/background editing
- shape and text tools
- hover previews
- palette manipulation
- true-color picker
- layers
- custom fonts/tilesets
- asset browsing
- skinnable UI
- many exports

UI implication: a modern dockable GUI is appropriate for the expanded feature set.

## Durdraw — terminal + animation

Durdraw combines:

- frame animation
- 16/256 colors
- Unicode and CP437
- terminal mouse
- custom themes
- brushes
- HTML/mIRC output
- DOS ANSI viewing
- playback

Its UI exposes keyboard commands and menus while retaining a canvas-first terminal workflow. [Durdraw](https://github.com/durdraw/durdraw)

UI implication: **animation belongs in the same document/canvas system**, and terminal operation should be a workspace/preset rather than a separate editor.

## Modern web ANSI editors — progressive disclosure

Modern browser editors demonstrate a deliberately simple default surface:

- CP437
- 16-color palette
- basic drawing tools
- selection
- undo/redo
- infinite canvas
- local import
- ANSI/PNG export

This is an excellent model for the Phase 1 default experience.

---

# 2. Overall UI Philosophy

Use four levels:

```text
LEVEL 1  Everyday Editor
   ↓
LEVEL 2  Advanced Art Studio
   ↓
LEVEL 3  Format / Animation / Collaboration Lab
   ↓
LEVEL 4  Preservation / Emulation / Research Lab
```

A user drawing an 80×25 ANSI screen should never need to see the Phase 4 machinery.

---

# 3. Recommended Primary Layout

```text
┌────────────────────────────────────────────────────────────────────────────┐
│ File Edit Draw Text Select Colors Fonts Animation View Tools Help          │
├───────┬──────────────────────────────────────────────────────┬─────────────┤
│ TOOL  │                                                      │ CONTEXT     │
│ RAIL  │                                                      │ PANEL       │
│       │                                                      │             │
│ Draw  │                       CANVAS                         │ Properties  │
│ Text  │                                                      │ Layers      │
│ Line  │                                                      │ Palette     │
│ Box   │                                                      │ Charset     │
│ Fill  │                                                      │ SAUCE       │
│ Select│                                                      │ Inspector   │
│ ...   │                                                      │             │
├───────┴──────────────────────────────────────────────────────┴─────────────┤
│ CHARACTER / COLOR / BRUSH STRIP                                            │
├────────────────────────────────────────────────────────────────────────────┤
│ TIMELINE — collapsed unless animation is active                            │
├────────────────────────────────────────────────────────────────────────────┤
│ STATUS: tool | char | FG/BG | x/y | canvas | frame | format | font | zoom │
└────────────────────────────────────────────────────────────────────────────┘
```

The **canvas must occupy most of the screen**.

---

# 4. Command Bar

Top-level menus:

```text
File  Edit  Draw  Text  Select  Colors  Fonts
Animation  View  Tools  Window  Help
```

### File

```text
New
Open
Save
Save As
Import
Export
Recent
Convert
Batch Convert
Document Properties
Preservation Package
```

### Draw

```text
Pencil
Line
Box
Fill
Rectangle
Ellipse
Brush
Stamp
Pattern
Box-Character Mode
Half-Block Mode
```

### Text

```text
Text Tool
Font
Character Set
Character Browser
Alignment
Justification
Text Effects
```

### Select

```text
Select
Copy
Cut
Paste
Move
Duplicate
Flip
Mirror
Rotate
Scale
Mask
```

### Colors

```text
Foreground
Background
Palette
Palette Editor
Color Picker
Recolor
Gradient
Dither
Color Mask
```

### Animation

```text
Timeline
New Frame
Duplicate Frame
Delete Frame
Frame Range
Playback
Timing
Transitions
ANSI Timing
Export Animation
```

### Tools

```text
Font Manager
Glyph Editor
SAUCE Editor
Format Inspector
ANSI Inspector
Reference Image
Art Browser
Compatibility
Diagnostics
Preservation
Collaboration
```

---

# 5. Left Tool Rail

Only high-frequency tools should remain permanently visible:

```text
Pencil
Line
Box
Fill
Text
Select
Eyedropper
Brush
Stamp
Undo
Redo
```

Additional tools should appear through flyouts, customization, keyboard shortcuts, or the command palette.

**Do not put 30–50 icons on the permanent toolbar.**

---

# 6. Canvas Modes

The same canvas should support:

### Authentic

Render exactly according to a selected historical/terminal target.

### Editing

Show:

- cell grid
- cursor
- selection
- guides
- rulers
- character boundaries

### Preview

Hide editing overlays.

### Terminal Playback

Actually render/execute the ANSI stream.

### Split

```text
┌────────────────────┬────────────────────┐
│ SOURCE / ANSI      │ RENDERED OUTPUT    │
└────────────────────┴────────────────────┘
```

---

# 7. Right Context Panel

Use a tabbed/collapsible inspector:

```text
Properties
Layers
Palette
Charset
Brushes
Animation
SAUCE
Inspector
Formats
```

Only the relevant panel needs to be open.

---

# 8. Properties Panel

Change it according to context.

### No selection

```text
Document
Width
Height
Format
Font
Palette
Encoding
```

### Cell

```text
Character
Code point
Encoding
Foreground
Background
Attributes
```

### Selection

```text
Width
Height
Character count
Color count
Transform
Transparency
```

### Frame

```text
Frame
Duration
FPS
Range
Delta/full-frame
```

---

# 9. Character / Glyph Panel

```text
┌───────────────────────────┐
│ CHARACTER SET             │
├───────────────────────────┤
│ ░ ▒ ▓ █ ▄ ▀ ▌ ▐ ...      │
│                           │
│ Selected: █               │
│ CP437: 219                │
│ Unicode: █               │
├───────────────────────────┤
│ Charset: CP437            │
│ Font: IBM VGA             │
│                           │
│ [Browse] [Edit] [Import]  │
└───────────────────────────┘
```

Support:

- CP437
- Unicode
- PETSCII
- Amiga sets
- ATASCII
- custom fonts
- TheDraw fonts
- iCE fonts
- XBIN fonts
- character remapping

---

# 10. Palette Panel

```text
PALETTE

█ █ █ █ █ █ █ █
█ █ █ █ █ █ █ █

FG ███
BG ███

Palette: VGA
Mode: 16 colors

[Edit] [Load] [Save]
[Remap] [Optimize]
```

Support:

- CGA
- EGA
- VGA
- iCE
- XBIN
- xterm 256
- custom 16-color
- RGB

Historical profiles should be able to **lock the palette**.

---

# 11. Brush Panel

Brushes may contain:

- characters
- colors
- transparency
- multiple cells
- patterns
- animation frames

```text
BRUSHES

[ Pencil ]
[ █ ]
[ ░▒▓ ]

Current: 7 × 3
Transparent background

[Create] [Edit] [Save] [Library]
```

---

# 12. Font Manager

Make this a full workspace:

```text
┌─────────────────────────────────────────┐
│ FONT MANAGER                            │
├───────────────┬─────────────────────────┤
│ Font Library  │ Character Preview       │
│ IBM VGA       │                         │
│ TheDraw       │  glyphs...              │
│ iCE           │                         │
│ PETSCII       │                         │
│ Amiga         │                         │
├───────────────┴─────────────────────────┤
│ Width / Height / Code page / Source     │
│ [Edit Glyphs] [Import] [Export]         │
└─────────────────────────────────────────┘
```

The glyph editor should directly edit bitmap characters.

---

# 13. Selection UI

After selecting an area, expose contextual operations:

```text
SELECTION

120 × 20 cells

Move   Copy   Cut
Fill   Clear  Duplicate

Flip H   Flip V
Rotate   Scale
Mirror   Mask

Save as Brush
Save as Pattern
```

This preserves TheDraw's powerful block workflow without permanent clutter.

---

# 14. Animation UI

Animation should be a **collapsible bottom workspace**.

Default:

```text
┌─────────────────────────────────────────────────────────────┐
│ CANVAS                                                      │
├─────────────────────────────────────────────────────────────┤
│ ▶ ◀ ▶ + duplicate delete | Frame 3/12 | FPS 8 | Range 1–12│
├─────────────────────────────────────────────────────────────┤
│ ▣ │ ▣ │ ▣ │ ▣ │ ▣ │ ▣ │ ▣ │ ▣ │ ▣ │ ▣ │ ▣ │ ▣            │
└─────────────────────────────────────────────────────────────┘
```

Animation should support:

- add/delete/duplicate/reorder frames
- thumbnails
- frame duration
- play/pause/stop
- loop/reverse
- onion skin
- frame copy/paste
- region copy between frames
- hold/blank/transition frames

---

# 15. Two Animation Timing Models

The UI should explicitly distinguish:

## Modern

```text
Frame 1 — 100 ms
Frame 2 — 100 ms
Frame 3 — 250 ms
```

## Historical ANSI

```text
Output timing
Baud simulation
ANSI delays
Cursor movement timing
Terminal rendering
```

Use:

```text
Playback:
(●) Modern frame timing
( ) Historical ANSI timing
```

Support historical rates such as 300, 1200, 2400 and 9600 baud plus custom timing.

---

# 16. Animation Range Editing

Expose a range:

```text
Current frame: 8
Edit range: 3–12
[Apply operation to range]
```

Operations can include drawing, movement, recoloring, replacement, flipping, deletion, duplication and effects.

---

# 17. Layers + Frames

Layers and animation should be orthogonal:

```text
Document
 ├── Layer 1
 │    ├── Frame 1
 │    ├── Frame 2
 │    └── Frame 3
 ├── Layer 2
 │    ├── Frame 1
 │    ├── Frame 2
 │    └── Frame 3
 └── Layer 3
```

The UI should show this through separate **Layers** and **Timeline** panels rather than one giant hierarchy.

---

# 18. Layer Panel

```text
LAYERS

☑ Background
☑ Artwork
☑ Text
☑ Effects
☐ Reference

[+] Add
[▣] Duplicate
[↑] [↓]
[Lock]
[Visibility]
```

Historical modes can disable or hide layers because they are editor constructs rather than legacy-format features.

---

# 19. Reference Image Tool

Support:

- image import
- opacity
- scaling
- rotation
- position
- crop
- lock
- hide

Reference images should not export unless deliberately rasterized/converted.

This is especially useful for recreating ANSI from screenshots.

---

# 20. Format Inspector

Example:

```text
FORMAT INSPECTOR

Format: ANSI
Encoding: CP437
Width: 80
Height: 50
Colors: 16
SAUCE: Present
Control sequences: 147
Unknown sequences: 0
Font: IBM VGA
Lossless: YES
```

For unsupported output:

```text
WARNING

This document contains:
  24-bit color
  transparency
  Unicode glyphs

ANSI export will lose:
  RGB precision
  transparency
  unsupported glyphs
```

---

# 21. ANSI Escape Inspector

Phase 4 specialist view:

```text
BYTE / SEQUENCE VIEW

000001  ESC [ 1 ; 34 m
000007  H
000008  e
000009  l
000010  l
000011  o
000012  ESC [ 0 m
```

Selecting a sequence highlights its visual effect.

This provides a bridge between:

```text
raw ANSI ↔ visual canvas
```

---

# 22. Source / Render Split

A preservation workspace should allow:

```text
┌──────────────────────────┬──────────────────────────┐
│ RAW SOURCE               │ RENDERED                │
│ ESC[1;31m████            │ ████████████            │
│ ESC[0m...                │                          │
└──────────────────────────┴──────────────────────────┘
```

Selecting either side highlights the corresponding visual/source region.

---

# 23. Historical Target Profile

Make the active target visible:

```text
TARGET: TheDraw 4.63
──────────────────────
Canvas: 80 × 100
Colors: VGA 16
Charset: CP437
Font: TheDraw TDF
Animation: ANSI
Timing: terminal
```

Profiles can include:

- CGA
- EGA
- VGA
- DOS ANSI
- TheDraw
- ACiDDraw
- iCE
- XBIN
- PCBoard
- RIP
- mIRC
- xterm 256
- Unicode terminal
- modern RGB

Profiles should control renderer, palette, font, constraints, format capabilities, export rules and warnings.

---

# 24. Authentic / Extended / Modern Modes

Use a simple document-target distinction:

```text
MODE

● Authentic
○ Extended
○ Modern
```

### Authentic

Only features representable by the target environment.

### Extended

Allows editor-only conveniences such as layers, guides, reference images and metadata while preserving an export target.

### Modern

Allows Unicode, RGB, 256 colors and modern terminal features.

This prevents historical constraints from making the entire editor frustrating.

---

# 25. Preservation Workspace

```text
PRESERVATION

Source
  filename
  checksum
  source application
  version
  OS
  date

Format
  detected format
  encoding
  palette
  font

Metadata
  SAUCE
  comments

Diagnostics
  [Validate]
  [Analyze]
  [Compare]

Provenance
  [View history]
  [Add source]
  [Record conversion]

Reproduction
  [Historical profile]
  [Render authentic]
```

---

# 26. Provenance Graph

```text
                    original.ans
                         │
                    imported
                         │
                 ┌───────┴───────┐
                 │               │
              restored         edited
                 │               │
                 └───────┬───────┘
                         │
                     exported
                         │
                      final.ans
```

Each node should retain:

- hash
- date
- application/version
- format
- operator
- font
- palette
- conversion information

---

# 27. Diagnostics Workspace

```text
ANSI VALIDATION

✓ Valid escape sequences
✓ CP437-compatible
✓ SAUCE valid
✓ Dimensions supported

Warnings:
⚠ 3 non-standard sequences
⚠ 1 unknown control code

Loss on ANSI export:
⚠ Unicode glyphs: 14
⚠ RGB colors: 28
⚠ Transparency: 3 regions
```

Warnings should be clickable and highlight affected cells.

---

# 28. Compatibility Matrix

Before exporting:

```text
EXPORT TO:

ANSI       ████████░░  82%
XBIN       ██████████ 100%
iCE        █████████░  91%
Unicode    ██████████ 100%
PNG        ██████████ 100%
```

Then show exact unsupported features and proposed conversions.

---

# 29. Collaboration Workspace

Keep collaboration out of the default UI.

Open:

```text
Window → Collaboration
```

Then:

```text
COLLABORATION

Session: ANSI-ROOM-42
Host: user

Participants
  ● Alice
  ● Bob
  ● Carol

Canvas
  Alice — x=42 y=18
  Bob   — x=11 y=31

[Invite] [Lock Region] [Sync] [Save Session]
```

This reflects PabloDraw/TundraDraw without making networking part of everyday drawing.

---

# 30. Art Browser / Asset Library

ATE's art-pack browsing concept should become a general library:

```text
┌──────────────────────────────────────────┐
│ ART LIBRARY                              │
├─────────────┬────────────────────────────┤
│ Artpacks    │ Preview                    │
│ Fonts       │                            │
│ Palettes    │       ANSI ART             │
│ Brushes     │                            │
│ Characters  │                            │
│ Templates   │                            │
├─────────────┴────────────────────────────┤
│ Metadata / SAUCE / Source                │
└──────────────────────────────────────────┘
```

It can eventually integrate local preservation collections.

---

# 31. Command Palette

A universal command palette is essential once the feature set becomes large:

```text
> flip horizontal
> import TDF font
> show SAUCE
> add frame
> convert to XBIN
> historical VGA mode
> open ANSI inspector
```

This keeps Phase 3/4 functionality discoverable without turning the main UI into a control panel.

---

# 32. Keyboard-First Operation

The editor must remain fully usable without a mouse.

Core actions:

```text
Arrows       Move cursor
Shift+Arrows Select
Ctrl+C       Copy
Ctrl+X       Cut
Ctrl+V       Paste
Ctrl+Z       Undo
Ctrl+Y       Redo
F-keys       Character groups / tools
```

Historical profiles can use TheDraw-like shortcuts; modern profiles can use customizable bindings.

---

# 33. Bottom Status Strip

Always expose the important state:

```text
DRAW | █ | FG:15 BG:1 | x:42 y:18 | 80×50 |
Frame 3/12 | 8 FPS | VGA | CP437 | 100%
```

This is the modern equivalent of the compact state information used by terminal editors.

---

# 34. Hover / Preview System

Borrow the strongest modern editing idea: preview before committing.

Use it for:

- recolor
- flip
- rotate
- palette swap
- font change
- dithering
- conversion
- resize

Workflow:

```text
Original → hover preview → click to commit
```

---

# 35. User UI Presets

## Beginner

```text
Canvas
Basic tools
Character palette
Color palette
Save/Open
Undo
```

## ANSI Artist

Adds:

```text
Font
Brush
Palette
Selection
Advanced drawing
Animation
```

## Power User

Adds:

```text
Layers
Timeline
Format tools
ANSI inspector
Batch conversion
Command palette
```

## Preservationist

Adds:

```text
Format inspector
SAUCE
Diagnostics
Provenance
Historical profiles
Raw source
Round-trip analysis
```

## Developer / Format Researcher

Adds:

```text
Byte inspector
Parser state
Format structures
Fuzzing
Capability matrix
Conversion diagnostics
```

These are UI presets, not separate applications.

---

# 36. Responsive Strategy

Desktop:

```text
left tools + canvas + right inspector
```

Small screens:

```text
canvas
+
bottom toolbar
+
slide-over panels
```

Panels become drawers rather than disappearing.

---

# 37. Terminal UI

A terminal version should focus on:

```text
Canvas
Character palette
Color palette
Selection
Tools
Animation
File
Help
```

and rely heavily on:

- keyboard
- function keys
- menus
- mouse where available

This follows Durdraw's strengths while sharing the same document model.

---

# 38. Browser/PWA UI

Use the same conceptual structure:

```text
top command bar
left tools
canvas
right inspector
bottom animation timeline
```

but hide specialist panels until invoked.

Offline-first behavior should be first-class.

---

# 39. What Should NOT Be Permanent UI

Do not permanently display:

- SAUCE fields
- format-conversion controls
- byte inspector
- provenance
- RIP commands
- collaboration controls
- historical emulation settings
- palette optimizer
- font editor
- diagnostic reports
- dozens of export formats

These are important capabilities, but not everyday controls.

---

# 40. Three Most Important Persistent Controls

Always visible:

```text
1. CURRENT TOOL
2. CURRENT CHARACTER / BRUSH
3. CURRENT COLORS
```

When relevant:

```text
4. CURRENT FRAME
5. CURRENT FONT / CHARSET
6. CURRENT TARGET PROFILE
```

---

# 41. Default Workspace

First launch should resemble:

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ File Edit Draw Text Select Colors Fonts Animation View Tools Help        │
├────┬───────────────────────────────────────────────────────────┬─────────┤
│ ✎  │                                                           │CHARACTER│
│ ╱  │                                                           │         │
│ □  │                                                           │ ░▒▓█    │
│ ▣  │                       ANSI CANVAS                         │ FG/BG   │
│ T  │                                                           │         │
│ ▦  │                                                           │ PALETTE │
│ ◉  │                                                           │         │
│ B  │                                                           │         │
├────┴───────────────────────────────────────────────────────────┴─────────┤
│ Tool: Pencil | █ | FG 15 | BG 1 | x 42 y 18 | VGA | CP437 | 100%       │
└──────────────────────────────────────────────────────────────────────────┘
```

No timeline, layers, diagnostics, provenance or format inspector by default.

---

# 42. When Animation Is Enabled

Expand only the bottom:

```text
┌──────────────────────────────────────────────────────────────────────────┐
│                              CANVAS                                      │
├──────────────────────────────────────────────────────────────────────────┤
│ ▶ ◀ ▶ + duplicate delete | Frame 3/12 | FPS 8 | Range 1–12             │
├──────────────────────────────────────────────────────────────────────────┤
│ ▣ ▣ ▣ ▣ ▣ ▣ ▣ ▣ ▣ ▣ ▣ ▣                                                  │
└──────────────────────────────────────────────────────────────────────────┘
```

Animation remains subordinate until needed.

---

# 43. When Preservation Mode Is Enabled

Transform the right context panel:

```text
┌───────────────────────┐
│ PRESERVATION          │
├───────────────────────┤
│ Format                │
│ ANSI / CP437 / 80×50 │
│ VGA                   │
│                       │
│ SAUCE                 │
│ ✓ valid               │
│                       │
│ Diagnostics           │
│ ✓ 147 sequences       │
│ ⚠ 2 unknown           │
│                       │
│ Target                │
│ TheDraw 4.63          │
│                       │
│ Provenance             │
│ Original → edited     │
└───────────────────────┘
```

The artwork itself does not move.

---

# 44. When Format Inspection Is Enabled

Use a split inspector:

```text
┌───────────────────────────────┬────────────────────────────────┐
│ SOURCE                        │ CANVAS                         │
│ ESC [ 1 ; 31 m               │ ███████████                    │
│ ESC [ 0 m                    │                                │
└───────────────────────────────┴────────────────────────────────┘
```

---

# 45. Recommended UI State Model

```text
App
 ├── Workspace
 │    ├── CanvasView
 │    ├── ToolRail
 │    ├── ContextPanel
 │    ├── Timeline
 │    └── StatusBar
 │
 ├── Document
 │    ├── Canvas
 │    ├── Layers
 │    ├── Frames
 │    ├── Palette
 │    ├── Charset
 │    └── Metadata
 │
 ├── TargetProfile
 │
 └── Services
      ├── Importers
      ├── Exporters
      ├── Renderer
      ├── Validator
      ├── Collaboration
      └── Preservation
```

This is preferable to building each roadmap phase as a separate UI subsystem.

---

# 46. Feature-to-UI Mapping

| Capability | Primary UI |
|---|---|
| Basic drawing | Left tool rail |
| Character selection | Character panel |
| Colors | Palette panel |
| Fonts | Font/Character workspace |
| Selection | Canvas overlay + context panel |
| Layers | Layers panel |
| Animation | Timeline |
| ANSI timing | Timeline / playback inspector |
| Legacy formats | File / Format workspace |
| SAUCE | Metadata panel |
| ANSI escape inspection | Inspector |
| Image conversion | Tools workspace |
| Collaboration | Collaboration workspace |
| Historical profiles | Target Profile |
| Diagnostics | Diagnostics workspace |
| Provenance | Preservation workspace |
| Raw source | Source/Render split |
| AI tools | Tools workspace |
| Batch conversion | Tools / Command Palette |
| Terminal mode | Workspace preset |
| Browser mode | Same workspace model, responsive |

---

# 47. Progressive Disclosure

The entire UI should follow:

```text
COMMON
  ↓
ADVANCED
  ↓
SPECIALIST
  ↓
RESEARCH
```

Example: Color

Beginner:

```text
FG / BG
```

Advanced:

```text
Palette
Recolor
Gradient
```

Specialist:

```text
CGA / EGA / VGA / iCE / XBIN
```

Research:

```text
RGB values
palette encoding
format constraints
export loss
```

One underlying color system; different presentation depths.

---

# 48. Historical UI as a Preset

Do not make a literal TheDraw clone the core architecture.

Instead:

```text
Modern Workspace
      │
      ├── Modern
      ├── TheDraw
      ├── ACiDDraw
      ├── iCE
      ├── Terminal
      └── Preservation
```

Historical workflows can therefore be preserved without trapping the application in a DOS-era interface.

---

# 49. Proposed UI Presets

## Modern

```text
Canvas
Tools
Character
Palette
Properties
```

## Classic ANSI

```text
Canvas
Character
Color
Font
Status
```

## TheDraw-like

```text
Full-screen canvas
Keyboard menus
Function-key character sets
Draw Mode
Animation
```

## Terminal

```text
Canvas
Compact menus
Keyboard shortcuts
Character groups
Timeline
```

## Animation

```text
Canvas
Timeline
Frames
Playback
Timing
```

## Format Lab

```text
Canvas
Format Inspector
Source
Renderer
Capability Matrix
```

## Preservation Lab

```text
Canvas
Diagnostics
Metadata
Provenance
Historical Target
Round-trip Test
```

---

# 50. The Most Important Design Separation

The UI must distinguish:

```text
WHAT I AM DRAWING
        ↓
Document / Canvas

HOW I AM DRAWING IT
        ↓
Tool / Font / Palette / Layer

HOW IT WILL BE REPRESENTED
        ↓
Format / Target Profile / Export
```

Example:

> "I want a 256-color Unicode canvas that can be exported as 16-color ANSI."

That should be completely normal.

Changing the export target should not require changing the document.

---

# 51. Animation Must Remain Independent

Likewise:

```text
Artwork
+
Animation
+
Target Format
```

should be separate dimensions.

The same document could therefore be:

```text
static ANSI
```

or:

```text
12-frame ANSI animation
```

or:

```text
12-frame Unicode animation
```

or:

```text
modern RGB animation
```

without changing the fundamental editor.

---

# 52. Final Information Architecture

```text
APP
│
├── FILE
│   ├── New/Open/Save
│   ├── Import/Export
│   ├── Convert
│   └── Preservation Package
│
├── EDIT
│   ├── Undo/Redo
│   ├── Clipboard
│   └── History
│
├── DRAW
│   ├── Pencil
│   ├── Shapes
│   ├── Fill
│   ├── Brush
│   └── Patterns
│
├── TEXT
│   ├── Text
│   ├── Font
│   ├── Charset
│   └── Glyph Editor
│
├── SELECT
│   ├── Transform
│   ├── Mask
│   └── Brush/Pattern creation
│
├── COLORS
│   ├── Palette
│   ├── Recolor
│   ├── Gradient
│   └── Dither
│
├── ANIMATION
│   ├── Timeline
│   ├── Frames
│   ├── Timing
│   └── Playback
│
├── VIEW
│   ├── Authentic
│   ├── Editing
│   ├── Preview
│   ├── Terminal
│   └── Split
│
├── TOOLS
│   ├── Format Inspector
│   ├── ANSI Inspector
│   ├── Art Browser
│   ├── Converter
│   ├── Diagnostics
│   ├── Collaboration
│   └── Preservation
│
├── TARGET
│   ├── DOS
│   ├── TheDraw
│   ├── iCE
│   ├── XBIN
│   ├── PCB
│   ├── RIP
│   ├── Terminal
│   └── Modern
│
└── HELP
    ├── Documentation
    ├── Format Reference
    ├── Keyboard Reference
    └── Historical Reference
```

---

# 53. Bottom-Line UI Recommendation

The application should **not** look like a giant control panel.

It should feel like:

> **A modern pixel-art editor whose pixels happen to be characters, colors, and terminal attributes.**

The default experience should be:

```text
              CHARACTER + COLOR
                     │
                     ▼
TOOLS ───────────► CANVAS ◄────────── PROPERTIES
                     │
                     ▼
                  TIMELINE
```

Everything else unfolds from those anchors.

The historical references become **sources of interaction patterns**, rather than competing UI designs:

- **TheDraw** → keyboard-first canvas, Draw Mode, fonts, transitions, animation
- **ACiDDraw** → large canvases, pages, VGA, text-layout tools
- **iCE** → custom glyphs, fonts, palette-centric ANSI work
- **PabloDraw** → desktop GUI, collaboration, RIP
- **ATE** → specialist font/palette/metadata ecosystem
- **Moebius** → graphical brush interaction
- **REXPaint** → layers, palettes, previews, assets, modern editing
- **Durdraw** → terminal UI, animation, keyboard/mouse hybrid, modern ANSI
- **modern web ANSI editors** → simplified progressive-disclosure workflow

The resulting architecture can scale from:

```text
"draw an 80×25 ANSI screen"
```

to:

```text
"reconstruct, inspect, animate, emulate, validate,
preserve, and export a historically accurate ANSI artifact."
```

without requiring separate applications or fundamentally different document models.

---

# 54. Core Design Principle

**Build one editor with one document model, but many progressively disclosed workspaces.**

Do not build:

```text
TheDraw mode
ACiDDraw mode
Moebius mode
Durdraw mode
Preservation mode
```

as separate internal applications.

Build:

```text
                 ONE DOCUMENT MODEL
                         │
                 ONE CANVAS ENGINE
                         │
        ┌────────────────┼────────────────┐
        │                │                │
      TOOLS          PANELS          WORKSPACES
        │                │                │
        └────────────────┼────────────────┘
                         │
                  TARGET PROFILES
                         │
        ┌────────────────┼────────────────┐
        │                │                │
     HISTORICAL       MODERN         PRESERVATION
```

This is the most realistic way to support **all four roadmap phases without making everyday editing overwhelming.**

---

# Reference Sources

- TheDraw — https://en.wikipedia.org/wiki/TheDraw
- ACiDDraw historical description — https://www.ansigallery.com/pack/out-0697/FILEDESC.TXT
- ATE — https://www.roysac.com/roy-tools/ansi-text-editor.html
- ATE beta feature history — https://www.roysac.com/blog/2014/06/ate-ansi-text-editor-1-0beta/
- PabloDraw — https://github.com/cwensley/pablodraw
- Moebius — https://github.com/blocktronics/moebius
- Moebius XBIN — https://github.com/hlotvonen/moebiusXBIN
- Durdraw — https://github.com/durdraw/durdraw
- REXPaint — https://www.gridsagegames.com/rexpaint/
- ANSIDRAW — https://ansidraw.com/
