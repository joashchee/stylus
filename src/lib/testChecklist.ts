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
  { id: "toggle-9px", section: "Viewer", label: "Tick 9-px spacing; the art widens by a ninth and box-drawing lines stay joined. Untick restores it", selector: '[data-testid="toggle-letter-spacing"]' },
  { id: "toggle-ice", section: "Viewer", label: "On art with blinking text, untick iCE colors; that text blinks about once a second. Tick it; the blink becomes a bright background", selector: '[data-testid="toggle-ice"]' },
  { id: "blink-reduced-motion", section: "Viewer", label: "Turn on Reduce motion (System Settings, Accessibility, Display), reopen that art with iCE off; the text shows steadily, no blinking" },
  { id: "toggle-aspect", section: "Viewer", label: "Tick Aspect ratio; the art stretches taller (by 1.2 at 8 px, 1.35 at 9 px)", selector: '[data-testid="toggle-aspect"]' },
  { id: "zoom", section: "Viewer", label: "Zoom: Fit width fills the panel, 100%/200%/300% show crisp square pixels, and the area scrolls", selector: '[data-testid="zoom-select"]' },
  { id: "long-art", section: "Viewer", label: "Open a very long ANSI (1000+ rows); it renders completely with no gaps between bands and scrolls smoothly" },
  { id: "ansi-trim", section: "Viewer", label: "Open a short ANSI (a few lines); the art ends at its last line, with no black 25-line screen below" },
  { id: "viewer-themes", section: "Viewer", label: "Switch themes with art open; the panels change, the art's colors never do" },
];
