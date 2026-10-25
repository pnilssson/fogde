export interface Rename {
  machine: string;
  from: string;
  to: string;
}

type Listener = (rename: Rename) => void;

const listeners = new Set<Listener>();

export function announce(rename: Rename) {
  for (const listener of listeners) {
    listener(rename);
  }
}

export function onRename(listener: Listener) {
  listeners.add(listener);

  return () => {
    listeners.delete(listener);
  };
}

export function moved(path: string, from: string, to: string) {
  const rest = path.slice(from.length);
  if (path.startsWith(from) && (rest === "" || /^[\\/]/.test(rest))) {
    return to + rest;
  }

  return undefined;
}
