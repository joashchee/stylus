/**
 * Form factors (docs/ui-design.md, "Phones, tablets and pens"): Stylus has
 * three discrete layouts, desktop, tablet and phone, each drawn in 8×16
 * cells, rather than one layout that reflows at every width. The profile is
 * a `data-form` attribute on <html>, which src/touch.css keys off, with
 * `data-touch` on the tablet and phone layouts and `data-drawers` where the context panel is a drawer over the canvas: a
 * phone, and a tablet held upright (under 120 columns, as docs/ui-design.md
 * has it for small windows).
 *
 * Automatic picks from the device, not the window: a touch-first screen is
 * a tablet, or a phone when its short side is under 600 CSS px (37 cells of
 * 16 px). A Mac, Windows or Linux window stays the desktop layout however
 * small it's made. The View menu's layouts choose one by hand, remembered.
 */
import { useEffect, useState } from "react";

export type FormFactor = "desktop" | "tablet" | "phone";
export type LayoutChoice = "auto" | FormFactor;

const LAYOUT_KEY = "stylus.layout";

/** iPadOS reports itself as a Mac; its touch points give it away. */
export const IS_MOBILE = /iPhone|iPad|iPod|Android/.test(navigator.userAgent) || (/Mac/.test(navigator.platform) && navigator.maxTouchPoints > 1);

export function detectFormFactor(): FormFactor {
  const touchFirst = IS_MOBILE || window.matchMedia("(pointer: coarse)").matches;
  if (!touchFirst) return "desktop";
  return Math.min(screen.width, screen.height) < 600 ? "phone" : "tablet";
}

export function loadLayout(): LayoutChoice {
  try {
    const stored = localStorage.getItem(LAYOUT_KEY);
    return stored === "desktop" || stored === "tablet" || stored === "phone" ? stored : "auto";
  } catch {
    return "auto";
  }
}

function storeLayout(choice: LayoutChoice) {
  try {
    if (choice === "auto") localStorage.removeItem(LAYOUT_KEY);
    else localStorage.setItem(LAYOUT_KEY, choice);
  } catch {
    // localStorage unavailable: the choice lasts until the app quits.
  }
}

function portrait() {
  return window.matchMedia("(orientation: portrait)").matches;
}

/** The layout in use, whether its panel is a drawer, the user's choice, and a way to change it. Sets <html data-form>. */
export function useFormFactor(): { form: FormFactor; drawers: boolean; choice: LayoutChoice; choose: (choice: LayoutChoice) => void } {
  const [choice, setChoice] = useState<LayoutChoice>(loadLayout);
  const [detected, setDetected] = useState<FormFactor>(detectFormFactor);
  const [upright, setUpright] = useState(portrait);
  const form = choice === "auto" ? detected : choice;
  const drawers = form === "phone" || (form === "tablet" && upright);

  useEffect(() => {
    // Turning the device is the one change that switches layouts by itself.
    const query = window.matchMedia("(orientation: portrait)");
    const update = () => {
      setUpright(query.matches);
      setDetected(detectFormFactor());
    };
    query.addEventListener("change", update);
    return () => query.removeEventListener("change", update);
  }, []);

  useEffect(() => {
    const root = document.documentElement;
    root.dataset.form = form;
    if (form === "desktop") delete root.dataset.touch;
    else root.dataset.touch = "";
    if (drawers) root.dataset.drawers = "";
    else delete root.dataset.drawers;
  }, [form, drawers]);

  function choose(next: LayoutChoice) {
    storeLayout(next);
    setChoice(next);
  }

  return { form, drawers, choice, choose };
}
