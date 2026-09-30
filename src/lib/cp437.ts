/**
 * The IBM PC character set (code page 437) as Unicode, for showing a
 * character code in the UI: the character picker, the F-key strip, the
 * status bar. The canvas itself draws from the art's own font; this is only
 * for labels. Codes 0–31 are the PC's glyphs, not control characters.
 */
const LOW = "\u0000☺☻♥♦♣♠•◘○◙♂♀♪♫☼►◄↕‼¶§▬↨↑↓→←∟↔▲▼";
const HIGH =
  "ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒáíóúñÑªº¿⌐¬½¼¡«»░▒▓│┤╡╢╖╕╣║╗╝╜╛┐└┴┬├─┼╞╟╚╔╩╦╠═╬╧╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀αßΓπΣσµτΦΘΩδ∞φε∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■ ";

export const CP437: readonly string[] = Array.from({ length: 256 }, (_, code) => {
  if (code < 32) return LOW[code] ?? " ";
  if (code < 127) return String.fromCharCode(code);
  if (code === 127) return "⌂";
  return HIGH[code - 128];
});

/** The Unicode for a CP437 code; codes past 255 (UTF-8 art) are their own. */
export function cp437Char(code: number): string {
  if (code >= 0 && code < 256) return code === 0 ? " " : CP437[code];
  return String.fromCodePoint(code);
}

/** The CP437 code for a typed character, or null when CP437 hasn't it. */
export function cp437Code(ch: string): number | null {
  if (ch.length === 0) return null;
  const cp = ch.codePointAt(0)!;
  if (cp >= 32 && cp < 127) return cp;
  const code = CP437.indexOf(ch);
  return code >= 32 ? code : null;
}

/**
 * The F-key character sets: ten keys each, typed with F1–F10 (the
 * TheDraw/PabloDraw convention). Stylus's own defaults, editable later.
 */
export const FKEY_SETS: readonly (readonly number[])[] = [
  [0xb0, 0xb1, 0xb2, 0xdb, 0xdf, 0xdc, 0xdd, 0xde, 0xfe, 0xfa], // shades and blocks
  [0xda, 0xbf, 0xc0, 0xd9, 0xc4, 0xb3, 0xc3, 0xb4, 0xc1, 0xc2], // single box
  [0xc9, 0xbb, 0xc8, 0xbc, 0xcd, 0xba, 0xcc, 0xb9, 0xca, 0xcb], // double box
  [0xd5, 0xb8, 0xd4, 0xbe, 0xcd, 0xb3, 0xc6, 0xb5, 0xcf, 0xd1], // double horizontal
  [0xd6, 0xb7, 0xd3, 0xbd, 0xc4, 0xba, 0xc7, 0xb6, 0xd0, 0xd2], // double vertical
  [0xc5, 0xce, 0xd8, 0xd7, 0xb3, 0xba, 0xc4, 0xcd, 0xfe, 0xf9], // crossings and lines
  [0x18, 0x19, 0x1a, 0x1b, 0x1e, 0x1f, 0x10, 0x11, 0x12, 0x1d], // arrows
  [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x0f, 0x0e, 0x0b], // symbols
  [0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xf7, 0xf0, 0xf1, 0xf2], // math
  [0x9b, 0x9c, 0x9d, 0x9e, 0x9f, 0xa8, 0xad, 0xae, 0xaf, 0xa9], // punctuation and money
];
