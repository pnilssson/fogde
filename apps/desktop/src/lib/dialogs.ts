export function inDialog(target: EventTarget | null) {
  return (
    target instanceof Element &&
    target.closest('[role="dialog"], [role="alertdialog"]') !== null
  );
}

export function inMenu(target: EventTarget | null) {
  return target instanceof Element && target.closest('[role="menu"]') !== null;
}

export function editable(target: EventTarget | null) {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable ||
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement)
  );
}
