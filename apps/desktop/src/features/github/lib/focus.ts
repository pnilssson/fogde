import type { KeyboardEvent } from "react";

export function moveFocus(event: KeyboardEvent<HTMLElement>) {
  if (event.key !== "ArrowDown" && event.key !== "ArrowUp") {
    return;
  }

  const rows = [
    ...event.currentTarget.querySelectorAll<HTMLElement>("[data-row]"),
  ];
  const index = rows.findIndex((row) => row === document.activeElement);
  const next = rows[event.key === "ArrowDown" ? index + 1 : index - 1];
  if (next) {
    event.preventDefault();
    next.focus();
  }
}
