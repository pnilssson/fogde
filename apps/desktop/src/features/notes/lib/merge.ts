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

export function merge(base: string, mine: string, theirs: string) {
  const ours = diff(base, mine);
  const their = diff(base, theirs);
  if (!ours || mine === theirs) {
    return theirs;
  }
  if (!their) {
    return mine;
  }

  const [first, second] =
    ours.from <= their.from ? [ours, their] : [their, ours];
  if (first.to > second.from || first.from === second.from) {
    return undefined;
  }

  return (
    base.slice(0, first.from) +
    first.insert +
    base.slice(first.to, second.from) +
    second.insert +
    base.slice(second.to)
  );
}
