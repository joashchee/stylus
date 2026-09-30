/**
 * App Testing checklist data (CLAUDE.md's Testing section), the same
 * mechanism as Diskette's. Lives only here, embedded in source, never a
 * separate spreadsheet or doc. Edit it in the same pass as the feature it
 * covers: add items when a feature lands, edit them when behavior changes,
 * and drop an item once it's been confirmed working.
 *
 * The overlay that renders this (components/AppTesting.tsx) is dev-only:
 * App.tsx only mounts it behind `import.meta.env.DEV`, so Vite drops it
 * from a production `tauri build`.
 */

export interface ChecklistItem {
  id: string;
  section: string;
  label: string;
  /** CSS selector for the "Go to app" jump target — omitted for a step with no single fixed control. */
  selector?: string;
}

export const CHECKLIST_DATA: ChecklistItem[] = [
  // Scaffold (build step 1, 2026-09-29)
  { id: "startup-dev-warning", section: "Scaffold", label: "First run (delete the in-development-accepted file in ~/Library/Application Support/com.ansiapps.stylus): before any window, a dialog says Stylus is still in development. I'll Be Back. quits with no window; OK opens the launch screen, and the next launch doesn't ask" },
  { id: "startup-ansi", section: "Scaffold", label: "Launch with nothing stored; the launch screen is blue in the VGA font from the first frame, then fades to the app" },
  {
    id: "theme-modern",
    section: "Scaffold",
    label: "Untick ANSIapps theme in the gear menu; the app switches to the modern dark look, and ticking it restores the blue DOS look",
    selector: '[data-testid="ansiapps-theme-toggle"]',
  },
  { id: "theme-persist-modern", section: "Scaffold", label: "Choose modern, relaunch; the launch screen is already dark, with no flash of blue" },
  { id: "about-credits", section: "Scaffold", label: "Gear → About Stylus lists the credits (VileR's VGA font, icy_tools) and the MIT license, readable in both themes" },
  { id: "core-version", section: "Scaffold", label: "With nothing open, the core line under Open shows stylus-core's version and the icy_tools revision", selector: '[data-testid="core-version"]' },

  // Viewer (build step 2, 2026-09-29)
  { id: "open-dialog", section: "Viewer", label: "Choose Open…; the dialog lists only art files. Pick an .ANS: a progress bar counts the rendering, then the art shows beside its SAUCE", selector: '[data-testid="open-button"]' },
  { id: "open-drop", section: "Viewer", label: "Drag an .ANS onto the window; a \"Drop to open\" box appears, and dropping opens it in place of the current art" },
  { id: "open-drop-wrong", section: "Viewer", label: "Drop a .jpg; an error lists the formats Stylus opens, and the current art stays" },
  { id: "open-untouched", section: "Viewer", label: "Open a file, then check it in Finder; its modified date hasn't changed" },
  { id: "formats", section: "Viewer", label: "Open one each of .ANS, .ASC/.NFO, .BIN, .XB, .ADF, .IDF, .TND, .PCB and .AVT; each renders, and XBin and ADF use their own font and palette" },
  { id: "sauce-shown", section: "Viewer", label: "Open art with SAUCE; the panel shows title, author, group, date, type, size, flags, font and comments, with accented group names correct (CP437)", selector: '[data-testid="sauce-panel"]' },
  { id: "sauce-none", section: "Viewer", label: "Open art without SAUCE; the panel says so and which format's defaults were used", selector: '[data-testid="sauce-panel"]' },
  { id: "toggle-9px", section: "Viewer", label: "View → 9-px Spacing; the art widens by a ninth and box-drawing lines stay joined, and the item is ticked. Choosing it again restores it", selector: '[data-testid="menu-view"]' },
  { id: "toggle-ice", section: "Viewer", label: "On art with blinking text, untick Colors → iCE Colors; that text blinks about once a second. Tick it; the blink becomes a bright background", selector: '[data-testid="menu-colors"]' },
  { id: "blink-reduced-motion", section: "Viewer", label: "Turn on Reduce motion (System Settings, Accessibility, Display), reopen that art with iCE off; the text shows steadily, no blinking" },
  { id: "toggle-aspect", section: "Viewer", label: "View → Aspect Ratio; the art stretches taller (by 1.2 at 8 px, 1.35 at 9 px)", selector: '[data-testid="menu-view"]' },
  { id: "zoom", section: "Viewer", label: "View → Fit Width fills the panel; ⌘+ and ⌘− step through 100% to 400% with crisp square pixels, ⌘0 is actual size, the status bar shows the zoom, and the area scrolls", selector: '[data-testid="menu-view"]' },
  { id: "long-art", section: "Viewer", label: "Open a very long ANSI (1000+ rows); it renders completely with no gaps between bands and scrolls smoothly" },
  { id: "ansi-trim", section: "Viewer", label: "Open a short ANSI (a few lines); the art ends at its last line, with no black 25-line screen below" },
  { id: "viewer-themes", section: "Viewer", label: "Switch themes with art open; the panels change, the art's colors never do" },

  // Image to ANSI (2026-09-30)
  { id: "i2a-tab", section: "Image to ANSI", label: "Choose Make at the right of the menu bar; Image to ANSI explains itself and offers Choose image…, and Art brings back the art that was open", selector: '[data-testid="tab-image-to-ansi"]' },
  { id: "i2a-choose", section: "Image to ANSI", label: "Choose image… and pick a photo; one progress bar counts the converters, and a card per converter fills in with its art, name, repository, license and size", selector: '[data-testid="choose-image"]' },
  { id: "i2a-drop", section: "Image to ANSI", label: "From the Art tab, drop a .png on the window; Stylus switches to Image to ANSI and converts it. Dropping an .ANS still opens it in the Art tab" },
  { id: "i2a-labels", section: "Image to ANSI", label: "Every card's name is different, and \"How it's run here\" shows its settings, what the port changes and the commit it was ported from" },
  { id: "i2a-save-one", section: "Image to ANSI", label: "Save .ANS… on a card; the dialog suggests <image>-<converter>.ans, and the saved file opens in the Art tab with SAUCE naming the converter in its comments" },
  { id: "i2a-save-all", section: "Image to ANSI", label: "Save all… into an empty folder; one .ANS per converter appears. Do it again into the same folder; nothing is replaced, and an error names the files left alone", selector: '[data-testid="save-all"]' },
  { id: "i2a-transparency", section: "Image to ANSI", label: "Convert a PNG with transparency and a tiny image (a 16×16 icon); every converter still finishes with art or a readable error" },
  { id: "i2a-untouched", section: "Image to ANSI", label: "After converting, check the image in Finder; its modified date hasn't changed" },
  { id: "i2a-new-image", section: "Image to ANSI", label: "Convert a second image while the first is still converting; the old cards give way to the new image's, with no mix of the two" },
  { id: "i2a-about", section: "Image to ANSI", label: "Gear → About Stylus lists every converter with its copyright, each license opens in full, and the ported libraries' licenses open too, readable in both themes" },
  { id: "i2a-themes", section: "Image to ANSI", label: "Switch themes with results showing; cards, tabs and labels stay readable, and the art's colors never change" },

  // Viewer finishing touches (Phase 1a, 2026-09-30)
  { id: "amiga-font", section: "Viewer", label: "Open an Amiga .ASC or .TXT without SAUCE; the View menu lists the fonts, and Font: Amiga Topaz 1+ redraws it in the Amiga font. Open another .ASC; it opens in the font chosen last", selector: '[data-testid="menu-view"]' },
  { id: "utf8-text", section: "Viewer", label: "Open a UTF-8 text file with block characters (▀ ▄ █); they show, not blanks" },

  // Editor, first slice (Phase 1c, 2026-09-30)
  { id: "edit-new", section: "Editor", label: "New… offers 80×25, 80×50, 132×25 and 160×25 or any size, and iCE colors; Create gives a black canvas named untitled-1.ans with the cursor at the top left", selector: '[data-testid="menu-file"]' },
  { id: "edit-type", section: "Editor", label: "Click the canvas and type; letters appear in the current colors and the cursor moves right. Arrows, Home/End and Page Up/Down move it; Backspace and Delete clear cells", selector: '[data-testid="art-canvas"]' },
  { id: "edit-grow", section: "Editor", label: "On the last row, press Enter or Down; a row is added and the cursor moves into it. Undo takes the row away" },
  { id: "edit-fkeys", section: "Editor", label: "F1–F10 type the strip's characters (on a laptop, with fn), clicking a strip key types it too, and ⌃← ⌃→ or the arrows change the set", selector: '[data-testid="fkey-strip"]' },
  { id: "edit-pencil", section: "Editor", label: "Pencil: drag quickly across the canvas; an unbroken line of the current character and colors. Right-drag erases to spaces in the background color", selector: '[data-testid="tool-pencil"]' },
  { id: "edit-paint-mode", section: "Editor", label: "Paints: Colors only recolors cells and keeps their characters; Character only keeps their colors", selector: '[data-testid="paint-mode"]' },
  { id: "edit-pick", section: "Editor", label: "Pick (or Option-click with any tool) on a cell; the current character, foreground and background become that cell's", selector: '[data-testid="tool-pick"]' },
  { id: "edit-colors", section: "Editor", label: "Colors panel: click sets the foreground (F), right-click or Shift-click the background (B). With iCE off, only the first eight can be backgrounds", selector: '[data-testid="color-panel"]' },
  { id: "edit-chars", section: "Editor", label: "Character panel: all 256 characters; clicking one makes it the Pencil's character", selector: '[data-testid="panel-tab-character"]' },
  { id: "edit-undo", section: "Editor", label: "⌘Z undoes a whole pencil drag, or one typed character, at a time; ⇧⌘Z redoes. Edit → Undo and Redo grey out when there's nothing to undo or redo", selector: '[data-testid="menu-edit"]' },
  { id: "edit-status", section: "Editor", label: "The status bar shows the tool, character, colors, cursor position, and the cell under the pointer with its code and colors", selector: '[data-testid="status-bar"]' },
  { id: "edit-edited", section: "Editor", label: "After a change, the window title and the name in the status bar show • and the status bar says Edited; undoing back to the last save clears them" },
  { id: "save-as", section: "Editor", label: "Save As…: pick each format; the dialog says what it keeps or loses (e.g. Plain text keeps no colors, BIN needs an even width), then Choose where… saves", selector: '[data-testid="menu-file"]' },
  { id: "save-blocked", section: "Editor", label: "On an odd-width canvas, Save As BIN or XBin; the reason shows in red and the save button stays disabled" },
  { id: "save-sauce", section: "Editor", label: "Fill in title, author, group and a comment, save, then open the file; the SAUCE panel shows them, with today's date for new art and the original date for opened art" },
  { id: "save-replace", section: "Editor", label: "Open a file, change it, press ⌘S; the dialog says it will replace the file. Replace, change it again, ⌘S; it saves without asking" },
  { id: "save-roundtrip", section: "Editor", label: "Save the same art as .ANS, .XB, .BIN, .ADF and .IDF and open each; they look the same as the original" },
  { id: "png-export", section: "Editor", label: "File → Export PNG… (⇧⌘E); the dialog starts from the view's 9-px and aspect settings and shows the pixel size, which changes as you tick them. Choose where…, save; a bar counts the rows, and the PNG opens in Preview matching the art", selector: '[data-testid="menu-file"]' },
  { id: "png-export-options", section: "Editor", label: "Export the same art at 8 px, 9 px and with aspect correction; the 9-px PNG is a ninth wider with box lines joined, the aspect one is 1.2× (1.35× at 9 px) taller with no uneven stripes in flat areas, and the view's settings are unchanged afterwards" },
  { id: "png-export-font", section: "Editor", label: "Export an XBin or Amiga text file whose font isn't 8 px wide; the letter-spacing choice is greyed out and names the font's width" },
  { id: "png-export-long", section: "Editor", label: "Export a very long ANSI (1000+ rows); the bar fills steadily, and the PNG is complete to the last row. Replace an existing PNG through the system dialog; it's replaced whole" },
  { id: "png-export-themes", section: "Editor", label: "Open the Export PNG dialog in both themes; the radio buttons, checkbox and size line are readable (( ) and (•) in the ANSIapps theme)" },
  { id: "save-discard", section: "Editor", label: "With unsaved changes, Open another file, choose New, or close the window; Stylus asks before discarding, and Keep editing keeps them" },

  // Tools and selection (Phase 1c, 2026-09-30)
  { id: "tool-keys", section: "Tools", label: "With the Pencil chosen, press L, R, B, H, S, F, I, E, T; each picks its tool. In Type, those letters type instead, and Esc goes back to the last tool", selector: '[data-testid="tool-line"]' },
  { id: "tool-line", section: "Tools", label: "Line: drag; a line follows the pointer as the art itself, and on letting go one ⌘Z removes it all", selector: '[data-testid="tool-line"]' },
  { id: "tool-rect", section: "Tools", label: "Rectangle: drag an outline, then tick Filled and drag again; the second is solid. Both undo in one step each", selector: '[data-testid="tool-rectangle"]' },
  { id: "tool-box", section: "Tools", label: "Box: drag; a box of ┌─┐│└┘ lines. Tick Double lines; ╔═╗. A one-row drag makes a straight line", selector: '[data-testid="tool-box"]' },
  { id: "tool-half", section: "Tools", label: "Half-block: drag slowly down one column; it paints half a cell at a time (▀ then █). Right-drag paints the background color. With iCE off, bright colors still come out right", selector: '[data-testid="tool-halfblock"]' },
  { id: "tool-fill", section: "Tools", label: "Fill: click inside a drawn box; the inside fills and the lines stop it. Right-click fills with blanks. One ⌘Z undoes it", selector: '[data-testid="tool-fill"]' },
  { id: "tool-eraser", section: "Tools", label: "Eraser: drag over art; cells become spaces in the background color", selector: '[data-testid="tool-eraser"]' },
  { id: "sel-drag", section: "Selection", label: "Select: drag a rectangle; a dashed outline shows it and the status bar gives its size. A click without dragging places the cursor and clears it", selector: '[data-testid="tool-select"]' },
  { id: "sel-keys", section: "Selection", label: "Shift+arrows select from the cursor; ⌘A selects everything; Esc deselects" },
  { id: "sel-copy-paste", section: "Selection", label: "Select, ⌘C, click elsewhere, ⌘V; the copy lands at the cursor and becomes the selection. ⌘X leaves blanks behind", selector: '[data-testid="selection-actions"]' },
  { id: "sel-transparent", section: "Selection", label: "Copy art with black space around it, paste over other art with Transparent ticked (or ⌥⌘V); the art underneath shows through the blanks", selector: '[data-testid="option-transparent"]' },
  { id: "sel-other-doc", section: "Selection", label: "Copy from one file, open or create another, paste; the cells come across with their colors" },
  { id: "sel-move", section: "Selection", label: "Drag inside a selection; the outline follows, and letting go moves the cells there, blanks behind. One ⌘Z puts it back" },
  { id: "sel-flip", section: "Selection", label: "Select a box with a ▌ beside it and Flip ↔; the box's corners and the half block mirror too. Flip ↕ turns ▀ into ▄" },
  { id: "sel-fill-clear", section: "Selection", label: "Fill paints the selection with the current character and colors; Clear (or Delete) blanks it" },
  { id: "sel-shortcuts-fields", section: "Selection", label: "In the Save As dialog, ⌘A and ⌘C/⌘V work on the text fields, not the art" },

  // Menu bar and command registry (Phase 1c, 2026-09-30)
  { id: "menu-open", section: "Menu bar", label: "Click File; its menu opens. Point at Edit, Draw, Select, Colors, View, Help in turn; each opens as the pointer arrives. Clicking outside or Esc closes it", selector: '[data-testid="menu-bar"]' },
  { id: "menu-keys", section: "Menu bar", label: "Press ⌥F (Option-F); File opens with its first item focused. Arrows move through items and across menus, Enter runs one, Esc closes and returns to the title" },
  { id: "menu-enabled", section: "Menu bar", label: "With nothing open, Save and the Edit items are greyed out; with a selection, Cut, Copy and the Select items light up; after Copy, Paste does" },
  { id: "menu-checks", section: "Menu bar", label: "Draw shows a tick by the current tool, Colors by the paint mode and iCE Colors, View by 9-px Spacing, Aspect Ratio and the zoom; choosing a tool in the rail moves the tick" },
  { id: "menu-shortcuts-shown", section: "Menu bar", label: "Items show their shortcuts in Mac style (⇧⌘S, ⌥⌘V), and Help → Keyboard Shortcuts lists every one, in both themes", selector: '[data-testid="menu-help"]' },
  { id: "menu-shortcuts-work", section: "Menu bar", label: "⌘N, ⌘O, ⌘S, ⇧⌘S, ⌘Z, ⇧⌘Z, ⌘A, ⌘C, ⌘X, ⌘V, ⌥⌘V, ⌘+, ⌘−, ⌘0 each do what their menu item does, and X swaps the colors while not typing" },
  { id: "menu-text-fields", section: "Menu bar", label: "In the Save As dialog's title field, ⌘A, ⌘C, ⌘V and ⌘Z act on the text, not the art; with no art selection, ⌘C still copies text selected elsewhere on the page" },
  { id: "menu-make", section: "Menu bar", label: "Switch to Make; the Edit, Draw and Select items grey out, and switching back to Art brings them back" },
  // macOS native menu bar (Phase 1c, 2026-09-30)
  { id: "native-menus", section: "Native menu", label: "The macOS menu bar has Stylus (About Stylus, Services, Hide, Quit), then File, Edit, Draw, Select, Colors, View, Window and Help, the same items as the in-window menus and in the same order" },
  { id: "native-state", section: "Native menu", label: "The macOS menu bar greys out and ticks the same items as the in-window menus: select cells and Cut/Copy light up; switch tool and the tick in Draw moves; switch to Make and the Edit, Draw and Select items grey out" },
  { id: "native-run", section: "Native menu", label: "Choosing an item in the macOS menu bar (Save As…, Flip Horizontal, Zoom In, Keyboard Shortcuts, About Stylus) does what the in-window item does, once" },
  { id: "native-keys-once", section: "Native menu", label: "With a selection on the art, ⌘C then ⌘V pastes one copy, ⌘Z undoes one step, and ⌘S opens Save once (no command runs twice); tool letters still work and the menu shows none" },
  { id: "native-text-fields", section: "Native menu", label: "In a text field (Save As title, Image to ANSI settings), ⌘A, ⌘C, ⌘X, ⌘V and ⌘Z edit the text, from the keyboard and from the macOS Edit menu; with a dialog open the menu bar's commands are greyed out" },
  { id: "native-quit", section: "Native menu", label: "With unsaved changes, ⌘Q (or Stylus → Quit) asks first; Cancel keeps the art open, discarding quits. ⌘W asks the same way" },
  { id: "recent-menu", section: "Recent and recovery", label: "Open two files, then File → Open Recent: both are there, newest first. Point at it, or press → on it; the submenu opens. Choose one; it opens. The same list is in the macOS File menu", selector: '[data-testid="menu-file"]' },
  { id: "recent-save", section: "Recent and recovery", label: "Save As a new file; it goes to the top of Open Recent. Quit and relaunch; the list is still there, and the empty screen shows it under Recent", selector: '[data-testid="recent-list"]' },
  { id: "recent-gone", section: "Recent and recovery", label: "Rename a file in Finder, then choose it from Open Recent; Stylus says it couldn't read it and it leaves the list. Clear Menu empties the list and greys out Open Recent" },
  { id: "recover-crash", section: "Recent and recovery", label: "Draw on a file, wait 10 seconds, then force-quit Stylus (⌥⌘Esc). Relaunch; Recover unsaved art names the file. Recover opens it with the drawing and • unsaved, and the file on disk is unchanged", selector: '[data-testid="recover-list"]' },
  { id: "recover-exact", section: "Recent and recovery", label: "Recover art with SAUCE, 9-px spacing, iCE off and blinking text; all of it comes back as it was, and Save As offers the SAUCE title and author" },
  { id: "recover-clean", section: "Recent and recovery", label: "Draw, wait 10 seconds, then Save (or quit and Discard changes); relaunch and nothing is offered. Later in the dialog keeps it for File → Recover Unsaved Art…; Discard removes it" },
  { id: "recover-themes", section: "Recent and recovery", label: "Check Open Recent's submenu, the Recent list on the empty screen and the Recover dialog in both themes" },
  { id: "edit-themes", section: "Editor", label: "Check the editor in both themes: menu bar and its menus, tool rail (with its options and Selection buttons), the selection outline, panels, strip, status bar and the Save and New dialogs are readable, and the art's colors never change" },
];
