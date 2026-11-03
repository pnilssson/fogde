import type { Terminal } from "@xterm/xterm";

import { appearance } from "@/features/terminals/lib/appearance";
import type { Display } from "@/generated/protocol";

export function colors(): Display {
  const styles = getComputedStyle(document.documentElement);
  const token = (name: string) => hex(styles.getPropertyValue(`--${name}`));
  const foreground = token("foreground");

  return { fg: foreground, bg: token("surface-1"), cursor: foreground };
}

export function followTheme(
  terminal: Terminal,
  element: HTMLElement,
  restyled: () => void,
  recolored: (display: Display) => void,
) {
  let shown = colors();
  const observer = new MutationObserver(() => {
    terminal.options = appearance(element);
    restyled();
    const next = colors();
    if (Object.values(next).join() !== Object.values(shown).join()) {
      shown = next;
      recolored(next);
    }
  });
  observer.observe(document.documentElement, { attributeFilter: ["class"] });

  return () => {
    observer.disconnect();
  };
}

let painter: CanvasRenderingContext2D | null | undefined;

function hex(color: string) {
  if (painter === undefined) {
    const canvas = document.createElement("canvas");
    canvas.width = 1;
    canvas.height = 1;
    painter = canvas.getContext("2d", { willReadFrequently: true });
  }
  if (!painter) {
    return "";
  }

  painter.clearRect(0, 0, 1, 1);
  painter.fillStyle = "transparent";
  painter.fillStyle = color.trim();
  painter.fillRect(0, 0, 1, 1);
  const [red = 0, green = 0, blue = 0, alpha = 0] = painter.getImageData(
    0,
    0,
    1,
    1,
  ).data;
  if (alpha === 0) {
    return "";
  }
  const channels = [red, green, blue].map((value) =>
    value.toString(16).padStart(2, "0"),
  );

  return `#${channels.join("")}`;
}
