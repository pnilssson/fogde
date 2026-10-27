import type { Text } from "@codemirror/state";
import type { SyntaxNode } from "@lezer/common";

import { linkClass, marks } from "@/features/notes/lib/preview";
import { wikilinkParts } from "@/features/notes/lib/wikilinks";

type Tag = "strong" | "em" | "s" | "code" | "span";

export type Inline =
  | string
  | {
      tag: Tag;
      className: string;
      attributes?: Record<string, string>;
      children: Inline[];
    };

const tags: Record<string, Tag> = {
  StrongEmphasis: "strong",
  Emphasis: "em",
  Strikethrough: "s",
  InlineCode: "code",
};

const skipped = new Set([
  "EmphasisMark",
  "StrikethroughMark",
  "CodeMark",
  "LinkMark",
  "WikiLinkMark",
  "LinkTitle",
  "URL",
]);

export function inline(doc: Text, node: SyntaxNode, from: number, to: number) {
  const found: Inline[] = [];
  let at = from;
  for (let child = node.firstChild; child; child = child.nextSibling) {
    if (child.to <= from || child.from >= to) {
      continue;
    }

    if (child.from > at) {
      found.push(doc.sliceString(at, child.from));
    }
    found.push(...element(doc, child));
    at = child.to;
  }

  if (at < to) {
    found.push(doc.sliceString(at, to));
  }

  return found;
}

function link(href: string, wiki: boolean, children: Inline[]): Inline {
  return {
    tag: "span",
    className: linkClass,
    attributes: wiki ? { "data-wikilink": href } : { "data-link": href },
    children,
  };
}

function element(doc: Text, node: SyntaxNode): Inline[] {
  const children = () => inline(doc, node, node.from, node.to);

  const tag = tags[node.name];
  if (tag) {
    return [{ tag, className: marks[node.name] ?? "", children: children() }];
  }

  switch (node.name) {
    case "Escape":
      return [doc.sliceString(node.from + 1, node.to)];
    case "URL":
      return node.parent?.name === "Link"
        ? []
        : [link(doc.sliceString(node.from, node.to), false, children())];
    case "Link": {
      const [open, close] = node.getChildren("LinkMark");
      const url = node.getChild("URL");
      if (!open || !close || !url) {
        return children();
      }

      return [
        link(
          doc.sliceString(url.from, url.to),
          false,
          inline(doc, node, open.to, close.from),
        ),
      ];
    }
    case "WikiLink": {
      const { target, label } = wikilinkParts(
        doc.sliceString(node.from + 2, node.to - 2),
      );
      return [link(target, true, [label])];
    }
    default:
      return skipped.has(node.name) ? [] : children();
  }
}

export function append(parent: HTMLElement, nodes: Inline[]) {
  for (const node of nodes) {
    if (typeof node === "string") {
      parent.append(node);
      continue;
    }

    const element = document.createElement(node.tag);
    element.className = node.className;
    for (const [name, value] of Object.entries(node.attributes ?? {})) {
      element.setAttribute(name, value);
    }
    append(element, node.children);
    parent.append(element);
  }
}
