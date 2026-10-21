import { frame } from "@/lib/frame";

export const shortcuts = {
  palette: "K",
  attention: "G",
} as const;

const mac = frame.platform === "macos";

function held(event: KeyboardEvent) {
  if (event.altKey || event.isComposing) {
    return false;
  }

  return mac
    ? event.metaKey && !event.ctrlKey && !event.shiftKey
    : event.ctrlKey && event.shiftKey && !event.metaKey;
}

export function pressed(event: KeyboardEvent, key: string) {
  return held(event) && event.code === `Key${key}`;
}

export function tileNumber(event: KeyboardEvent) {
  const digit = /^Digit([1-9])$/.exec(event.code)?.[1];

  return held(event) && digit ? Number(digit) : undefined;
}

export function keys(key: string) {
  return mac ? ["⌘", key] : ["Ctrl", "Shift", key];
}
