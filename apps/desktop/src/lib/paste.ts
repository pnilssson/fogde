export function bracket(text: string) {
  const clean = text
    .replace(/\r\n?/g, "\n")
    .replaceAll("\x1b[200~", "")
    .replaceAll("\x1b[201~", "");

  return `\x1b[200~${clean}\x1b[201~`;
}

export function pastePath(path: string) {
  return bracket(/^[\w./\\:~-]+$/.test(path) ? path : `"${path}"`);
}
