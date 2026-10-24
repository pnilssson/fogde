import type { ITerminalOptions } from "@xterm/xterm";

export function appearance(element: HTMLElement) {
  const styles = getComputedStyle(document.documentElement);
  const text = getComputedStyle(element);
  const token = (name: string) => styles.getPropertyValue(`--${name}`).trim();
  const size = Number.parseFloat(text.fontSize);

  return {
    fontFamily: text.fontFamily,
    fontSize: size,
    lineHeight: Number.parseFloat(text.lineHeight) / size,
    cursorStyle: "bar",
    cursorInactiveStyle: "outline",
    drawBoldTextInBrightColors: false,
    theme: {
      background: "#00000000",
      foreground: token("foreground"),
      cursor: token("foreground"),
      cursorAccent: token("surface-1"),
      selectionBackground: token("terminal-selection"),
      selectionInactiveBackground: token("terminal-selection"),
      black: token("terminal-black"),
      red: token("terminal-red"),
      green: token("terminal-green"),
      yellow: token("terminal-yellow"),
      blue: token("terminal-blue"),
      magenta: token("terminal-magenta"),
      cyan: token("terminal-cyan"),
      white: token("terminal-white"),
      brightBlack: token("terminal-bright-black"),
      brightRed: token("terminal-bright-red"),
      brightGreen: token("terminal-bright-green"),
      brightYellow: token("terminal-bright-yellow"),
      brightBlue: token("terminal-bright-blue"),
      brightMagenta: token("terminal-bright-magenta"),
      brightCyan: token("terminal-bright-cyan"),
      brightWhite: token("terminal-bright-white"),
    },
  } satisfies ITerminalOptions;
}
