import type { Note } from "@/generated/protocol";

function parts(path: string) {
  const windows = /^[a-z]:/i.test(path) || path.includes("\\");
  const resolved: string[] = [];

  for (const part of path.split(/[\\/]/)) {
    if (part === "" || part === ".") {
      continue;
    }
    if (part === "..") {
      resolved.pop();
      continue;
    }

    resolved.push(windows ? part.toLowerCase() : part);
  }

  return resolved;
}

function key(path: string) {
  return parts(path).join("/");
}

function withoutExtension(path: string) {
  return path.replace(/\.md$/i, "");
}

function closest(found: Note[], from: Note | undefined) {
  const here = from && key(from.folder);
  const ranked = found.toSorted(
    (a, b) =>
      Number(key(b.folder) === here) - Number(key(a.folder) === here) ||
      parts(a.path).length - parts(b.path).length ||
      a.path.localeCompare(b.path),
  );

  return ranked[0];
}

export function resolveWikilink(
  target: string,
  notes: Note[],
  from: Note | undefined,
) {
  const wanted = key(
    withoutExtension(target.split("#")[0] ?? ""),
  ).toLowerCase();
  if (wanted === "") {
    return undefined;
  }

  const found = notes.filter((note) => {
    const path = key(withoutExtension(note.path)).toLowerCase();

    return path === wanted || path.endsWith(`/${wanted}`);
  });

  return closest(found, from);
}

export function resolveHref(href: string, notes: Note[], from: Note) {
  let path: string;
  try {
    path = decodeURIComponent(href.split("#")[0] ?? "");
  } catch {
    return undefined;
  }
  if (!/\.md$/i.test(path)) {
    return undefined;
  }

  const wanted = key(`${from.folder}/${path}`);
  const relative = notes.find((note) => key(note.path) === wanted);

  return relative ?? resolveWikilink(path, notes, from);
}
