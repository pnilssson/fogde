export interface Change {
  from: number;
  to: number;
  insert: string;
}

export function diff(before: string, after: string): Change | undefined {
  if (before === after) {
    return undefined;
  }

  const shortest = Math.min(before.length, after.length);
  let start = 0;
  while (start < shortest && before[start] === after[start]) {
    start++;
  }

  let end = 0;
  while (
    end < shortest - start &&
    before[before.length - 1 - end] === after[after.length - 1 - end]
  ) {
    end++;
  }

  return {
    from: start,
    to: before.length - end,
    insert: after.slice(start, after.length - end),
  };
}

const limit = 4_000_000;

interface Hunk {
  from: number;
  to: number;
  lines: string[];
  mine: boolean;
}

export interface Merged {
  text: string;
  kept: boolean;
}

function ending(text: string) {
  return text.includes("\r\n") ? "\r\n" : "\n";
}

function split(text: string) {
  return text.match(/[^\n]*\n|[^\n]+$/g) ?? [];
}

function hunks(base: string[], other: string[], mine: boolean): Hunk[] {
  let start = 0;
  while (
    start < base.length &&
    start < other.length &&
    base[start] === other[start]
  ) {
    start++;
  }

  let end = 0;
  while (
    end < base.length - start &&
    end < other.length - start &&
    base[base.length - 1 - end] === other[other.length - 1 - end]
  ) {
    end++;
  }

  const before = base.slice(start, base.length - end);
  const after = other.slice(start, other.length - end);
  const width = after.length + 1;
  if ((before.length + 1) * width > limit) {
    return [{ from: start, to: start + before.length, lines: after, mine }];
  }

  const common = new Uint32Array((before.length + 1) * width);
  const shared = (i: number, j: number) => common[i * width + j] ?? 0;
  for (let i = before.length - 1; i >= 0; i--) {
    for (let j = after.length - 1; j >= 0; j--) {
      common[i * width + j] =
        before[i] === after[j]
          ? shared(i + 1, j + 1) + 1
          : Math.max(shared(i + 1, j), shared(i, j + 1));
    }
  }

  const found: Hunk[] = [];
  let open: Hunk | undefined;
  let i = 0;
  let j = 0;
  while (i < before.length || j < after.length) {
    const removed = before[i];
    const added = after[j];
    if (removed !== undefined && removed === added) {
      open = undefined;
      i++;
      j++;
      continue;
    }

    if (!open) {
      open = { from: start + i, to: start + i, lines: [], mine };
      found.push(open);
    }

    if (
      added !== undefined &&
      (removed === undefined || shared(i, j + 1) >= shared(i + 1, j))
    ) {
      open.lines.push(added);
      j++;
    } else {
      i++;
      open.to = start + i;
    }
  }

  return found;
}

function collide(a: Hunk, b: Hunk) {
  if (a.from < b.to && b.from < a.to) {
    return true;
  }

  return a.from === a.to && b.from === b.to && a.from === b.from;
}

function apply(base: string[], side: Hunk[], from: number, to: number) {
  const lines: string[] = [];
  let at = from;
  for (const hunk of side) {
    lines.push(...base.slice(at, hunk.from), ...hunk.lines);
    at = hunk.to;
  }
  lines.push(...base.slice(at, to));

  return lines.join("");
}

export function merge(base: string, mine: string, theirs: string): Merged {
  const original = split(base);
  const all = [
    ...hunks(original, split(mine), true),
    ...hunks(original, split(theirs), false),
  ].sort((a, b) => a.from - b.from || a.to - a.from - (b.to - b.from));

  const regions: Hunk[][] = [];
  for (const hunk of all) {
    const last = regions.at(-1);
    if (last?.some((other) => collide(other, hunk))) {
      last.push(hunk);
    } else {
      regions.push([hunk]);
    }
  }

  let text = "";
  let kept = false;
  let at = 0;
  for (const region of regions) {
    const from = Math.min(...region.map((hunk) => hunk.from));
    const to = Math.max(...region.map((hunk) => hunk.to));
    const ours = region.filter((hunk) => hunk.mine);
    const their = region.filter((hunk) => !hunk.mine);
    const own = apply(original, ours, from, to);
    const other = apply(original, their, from, to);
    text += original.slice(at, from).join("");
    if (ours.length === 0) {
      text += other;
    } else if (from === to && their.length > 0 && own !== other) {
      text += other.endsWith("\n")
        ? other + own
        : other + ending(original.join("") + other + own) + own;
    } else {
      text += own;
      kept ||= their.length > 0 && own !== other;
    }
    at = to;
  }
  text += original.slice(at).join("");

  return { text, kept };
}
