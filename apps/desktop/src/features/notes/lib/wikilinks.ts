import type { MarkdownConfig } from "@lezer/markdown";

const bracket = 91;
const closing = 93;
const bang = 33;
const newline = 10;

export const wikilinks: MarkdownConfig = {
  defineNodes: ["WikiLink", "WikiLinkMark"],
  parseInline: [
    {
      name: "WikiLink",
      before: "Link",
      parse(cx, next, pos) {
        if (
          next !== bracket ||
          cx.char(pos + 1) !== bracket ||
          cx.char(pos - 1) === bang
        ) {
          return -1;
        }

        for (let end = pos + 2; end < cx.end - 1; end++) {
          const char = cx.char(end);
          if (char === newline || char === bracket) {
            return -1;
          }
          if (char !== closing) {
            continue;
          }
          if (cx.char(end + 1) !== closing || end === pos + 2) {
            return -1;
          }

          return cx.addElement(
            cx.elt("WikiLink", pos, end + 2, [
              cx.elt("WikiLinkMark", pos, pos + 2),
              cx.elt("WikiLinkMark", end, end + 2),
            ]),
          );
        }

        return -1;
      },
    },
  ],
};

export function wikilinkParts(inner: string) {
  const pipe = inner.indexOf("|");

  return pipe < 0
    ? { target: inner.trim(), label: inner }
    : { target: inner.slice(0, pipe).trim(), label: inner.slice(pipe + 1) };
}
