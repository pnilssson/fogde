import { frame } from "@/lib/frame";

export const shortcuts = {
  palette: "K",
  attention: "G",
  tile: "T",
  close: "W",
  path: "P",
  settings: ",",
} as const;

export const arrows = [
  { code: "ArrowLeft", side: "left", glyph: "←" },
  { code: "ArrowRight", side: "right", glyph: "→" },
  { code: "ArrowUp", side: "top", glyph: "↑" },
  { code: "ArrowDown", side: "bottom", glyph: "↓" },
] as const;

const mac = frame.platform === "macos";

function held(event: KeyboardEvent, alt = false) {
  if (event.altKey !== alt || event.isComposing) {
    return false;
  }

  return mac
    ? event.metaKey && !event.ctrlKey && !event.shiftKey
    : event.ctrlKey && event.shiftKey && !event.metaKey;
}

export function pressed(event: KeyboardEvent, key: string) {
  return held(event) && event.code === code(key);
}

function code(key: string) {
  return key === "," ? "Comma" : `Key${key}`;
}

export function tileNumber(event: KeyboardEvent) {
  const digit = /^Digit([1-9])$/.exec(event.code)?.[1];

  return held(event) && digit ? Number(digit) : undefined;
}

export function arrowSide(event: KeyboardEvent) {
  if (!held(event, true)) {
    return undefined;
  }

  return arrows.find(({ code }) => code === event.code)?.side;
}

export function keys(key: string, alt = false) {
  if (mac) {
    return alt ? `⌘⌥${key}` : `⌘${key}`;
  }

  return alt ? `Ctrl+Shift+Alt+${key}` : `Ctrl+Shift+${key}`;
}
