import { syntaxTree } from "@codemirror/language";
import type { EditorState, Range } from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  type EditorView,
  ViewPlugin,
  type ViewUpdate,
} from "@codemirror/view";
import type { SyntaxNodeRef } from "@lezer/common";

import { indent, type Items, listItem } from "@/features/notes/lib/lists";
import { Rule } from "@/features/notes/lib/widgets";
import { wikilinkParts } from "@/features/notes/lib/wikilinks";

export const linkClass =
  "cursor-pointer text-primary underline underline-offset-2";

const hidden = Decoration.replace({});

const minor = "font-semibold text-muted-foreground";

const headings: Record<string, string> = {
  ATXHeading1: "type-display",
  ATXHeading2: "type-note-heading",
  ATXHeading3: "font-semibold",
  ATXHeading4: minor,
  ATXHeading5: minor,
  ATXHeading6: minor,
  SetextHeading1: "type-display",
  SetextHeading2: "type-note-heading",
};

export const marks: Record<string, string> = {
  Emphasis: "italic",
  StrongEmphasis: "font-bold",
  Strikethrough: "line-through",
  InlineCode: "rounded-sm bg-muted px-1 type-code",
};

const blocks: Record<string, string> = {
  FencedCode: "cm-code bg-muted type-code",
  CodeBlock: "cm-code bg-muted type-code",
  Blockquote: "cm-quote border-l-2 text-muted-foreground",
  Table: "type-code",
};

function activeLines(view: EditorView) {
  const active = new Set<number>();
  if (!view.hasFocus) {
    return active;
  }

  const { doc, selection } = view.state;
  for (const range of selection.ranges) {
    const last = doc.lineAt(range.to).number;
    for (let line = doc.lineAt(range.from).number; line <= last; line++) {
      active.add(line);
    }
  }

  return active;
}

function link(href: string, wiki: boolean) {
  return Decoration.mark({
    class: linkClass,
    attributes: wiki ? { "data-wikilink": href } : { "data-link": href },
  });
}

function decorate(
  state: EditorState,
  node: SyntaxNodeRef,
  editing: boolean,
  found: Range<Decoration>[],
) {
  const { doc } = state;
  const name = node.name;
  const text = (from: number, to: number) => doc.sliceString(from, to);
  const hide = (from: number, to: number) => {
    if (from < to && doc.lineAt(from).number === doc.lineAt(to).number) {
      found.push(hidden.range(from, to));
    }
  };

  const heading = headings[name];
  if (heading) {
    const first = doc.lineAt(node.from).number;
    const last = name.startsWith("Setext")
      ? doc.lineAt(node.to).number - 1
      : first;
    found.push(
      Decoration.line({ class: `${heading} cm-h${name.slice(-1)}` }).range(
        doc.line(first).from,
      ),
    );
    for (let line = first + 1; line <= last; line++) {
      found.push(
        Decoration.line({ class: heading }).range(doc.line(line).from),
      );
    }
  }

  const block = blocks[name];
  if (block) {
    const last = doc.lineAt(node.to).number;
    for (let line = doc.lineAt(node.from).number; line <= last; line++) {
      found.push(Decoration.line({ class: block }).range(doc.line(line).from));
    }
  }

  if (name === "FencedCode" || name === "CodeBlock") {
    found.push(
      Decoration.line({ class: "rounded-t-md" }).range(
        doc.lineAt(node.from).from,
      ),
      Decoration.line({ class: "rounded-b-md" }).range(
        doc.lineAt(node.to).from,
      ),
    );
  }

  const markClass = marks[name];
  if (markClass) {
    found.push(Decoration.mark({ class: markClass }).range(node.from, node.to));
  }

  if (name === "URL" && node.node.parent?.name !== "Link") {
    found.push(link(text(node.from, node.to), false).range(node.from, node.to));
  }

  if (editing) {
    return;
  }

  switch (name) {
    case "HeaderMark": {
      const end = Math.min(node.to + 1, doc.lineAt(node.from).to);
      if (node.from === doc.lineAt(node.from).from) {
        hide(node.from, end);
      }
      break;
    }
    case "QuoteMark": {
      const end = Math.min(node.to + 1, doc.lineAt(node.from).to);
      hide(node.from, end);
      break;
    }
    case "EmphasisMark":
    case "StrikethroughMark":
      hide(node.from, node.to);
      break;
    case "CodeMark": {
      const parent = node.node.parent?.name;
      if (parent === "InlineCode") {
        hide(node.from, node.to);
      } else if (parent === "FencedCode") {
        hide(node.from, doc.lineAt(node.from).to);
      }
      break;
    }
    case "HorizontalRule":
      found.push(
        Decoration.replace({ widget: new Rule() }).range(node.from, node.to),
      );
      break;
    case "Link": {
      const parts = node.node.getChildren("LinkMark");
      const url = node.node.getChild("URL");
      const [open, close] = parts;
      if (!open || !close || !url || open.from !== node.from) {
        break;
      }

      if (open.to < close.from) {
        hide(open.from, open.to);
        found.push(
          link(text(url.from, url.to), false).range(open.to, close.from),
        );
        hide(close.from, node.to);
      }
      break;
    }
    case "WikiLink": {
      const inner = text(node.from + 2, node.to - 2);
      const { target, label } = wikilinkParts(inner);
      const start = node.to - 2 - label.length;
      if (start < node.to - 2) {
        hide(node.from, start);
        found.push(link(target, true).range(start, node.to - 2));
        hide(node.to - 2, node.to);
      }
      break;
    }
  }
}

function build(view: EditorView) {
  const found: Range<Decoration>[] = [];
  const items: Items = new Map();
  const active = activeLines(view);
  const { state } = view;
  const { doc } = state;

  for (const { from, to } of view.visibleRanges) {
    syntaxTree(state).iterate({
      from,
      to,
      enter: (node) => {
        if (node.name === "ListItem") {
          listItem(state, node.node, active, items, found);
        }

        const first = doc.lineAt(node.from).number;
        const last = doc.lineAt(node.to).number;
        let editing = false;
        for (let line = first; line <= last && !editing; line++) {
          editing = active.has(line);
        }

        decorate(state, node, editing, found);
      },
    });
  }

  return Decoration.set([...found, ...indent(state, items)], true);
}

export const preview = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor(view: EditorView) {
      this.decorations = build(view);
    }

    update(update: ViewUpdate) {
      if (
        update.docChanged ||
        update.viewportChanged ||
        update.selectionSet ||
        update.focusChanged ||
        syntaxTree(update.startState) !== syntaxTree(update.state)
      ) {
        this.decorations = build(update.view);
      }
    }
  },
  { decorations: (plugin) => plugin.decorations },
);
