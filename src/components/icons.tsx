/**
 * Inline SVG icons, the same vocabulary as Diskette's
 * (src/components/icons.tsx there). currentColor throughout so every icon
 * themes for free. These are stand-ins: once the theme pack exists, Stylus
 * is the first app to use its ANSI icons instead (Diskette's
 * docs/stylus-notes.md).
 *
 * AppMarkIcon is the header's mark, the same drawing as the app icon
 * (src-tauri/icons/source-icon.svg is the master, run through `tauri icon`
 * for every platform size), cropped to its pixels. It's the one glyph with
 * fixed colors: a brand mark, not a themeable UI icon.
 */
import type { SVGProps } from "react";

type IconProps = SVGProps<SVGSVGElement>;

export function AppMarkIcon(props: IconProps) {
  return (
    <svg viewBox="0 0 260 280" aria-hidden="true" {...props}>
      <g fill="#ff55ff">
      <rect x="40" y="0" width="40" height="40" />
      <rect x="80" y="0" width="40" height="40" />
      <rect x="120" y="0" width="40" height="40" />
      <rect x="160" y="0" width="40" height="40" />
      <rect x="0" y="40" width="40" height="40" />
      <rect x="0" y="80" width="40" height="40" />
      <rect x="40" y="120" width="40" height="40" />
      <rect x="80" y="120" width="40" height="40" />
      <rect x="120" y="120" width="40" height="40" />
      <rect x="160" y="160" width="40" height="40" />
      <rect x="160" y="200" width="40" height="40" />
      <rect x="0" y="240" width="40" height="40" />
      <rect x="40" y="240" width="40" height="40" />
      <rect x="80" y="240" width="40" height="40" />
      <rect x="120" y="240" width="40" height="40" />
      </g>
      <rect x="220" y="266" width="40" height="14" fill="#ffffff" />
    </svg>
  );
}

export function FolderIcon(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" {...props}>
      <path
        fill="none"
        stroke="currentColor"
        strokeWidth={2}
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"
      />
    </svg>
  );
}

export function GearIcon(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" {...props}>
      <circle cx="12" cy="12" r="3" fill="none" stroke="currentColor" strokeWidth={2} />
      <path
        fill="none"
        stroke="currentColor"
        strokeWidth={2}
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
      />
    </svg>
  );
}

export function ChecklistIcon(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" {...props}>
      <rect x="4" y="3" width="16" height="18" rx="2" fill="none" stroke="currentColor" strokeWidth={2} />
      <path fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round" d="M8 8.5l1.5 1.5L12 7" />
      <path stroke="currentColor" strokeWidth={2} strokeLinecap="round" d="M14.5 9h3" />
      <path fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round" d="M8 14.5l1.5 1.5L12 13" />
      <path stroke="currentColor" strokeWidth={2} strokeLinecap="round" d="M14.5 15h3" />
    </svg>
  );
}

export function InfoIcon(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" {...props}>
      <circle cx="12" cy="12" r="9" fill="none" stroke="currentColor" strokeWidth={2} />
      <path stroke="currentColor" strokeWidth={2} strokeLinecap="round" d="M12 11v5.5" />
      <circle cx="12" cy="8" r="1" fill="currentColor" />
    </svg>
  );
}
