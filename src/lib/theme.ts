/**
 * UI theme: "ansiapps" (the old-school DOS look, see
 * docs/ansiapps-theme.md) or "modern". Stylus is the one ansiapps app with
 * the default flipped: it's the theme's own workshop, so it opens in the
 * theme and offers modern as the option (Diskette's docs/stylus-notes.md,
 * decided 2026-09-29). The choice is a `data-theme` attribute on <html>,
 * which src/ansiapps-theme.css keys off. index.html's inline script
 * applies the stored choice before first paint, so it must use the same
 * key and values, and assume ANSIapps when nothing is stored.
 */

export type Theme = "modern" | "ansiapps";

/** Also read by the inline script in index.html. */
const THEME_KEY = "stylus.theme";

export function loadTheme(): Theme {
  try {
    return localStorage.getItem(THEME_KEY) === "modern" ? "modern" : "ansiapps";
  } catch {
    return "ansiapps";
  }
}

export function applyTheme(theme: Theme) {
  if (theme === "ansiapps") document.documentElement.dataset.theme = "ansiapps";
  else delete document.documentElement.dataset.theme;
  try {
    localStorage.setItem(THEME_KEY, theme);
  } catch {
    // localStorage unavailable — the theme just won't persist across launches.
  }
}
