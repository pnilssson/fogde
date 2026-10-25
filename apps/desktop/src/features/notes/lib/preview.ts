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

import { Bullet, Checkbox, Rule } from "@/features/notes/lib/widgets";
import { wikilinkParts } from "@/features/notes/lib/wikilinks";

const linkClass = "cursor-pointer text-primary underline underline-offset-2";

const hidden = Decoration.replace({});

const headings: Record<string, string> = {
  ATXHeading1: "type-display",
  ATXHeading2: "type-title",
  ATXHeading3: "type-title",
  ATXHeading4: "font-semibold",
  ATXHeading5: "font-semibold",
  ATXHeading6: "font-semibold",
  SetextHeading1: "type-display",
  SetextHeading2: "type-title",
};

const marks: Record<string, string> = {
  Emphasis: "italic",
  StrongEmphasis: "font-bold",
  Strikethrough: "line-through",
  InlineCode: "rounded-sm bg-muted px-1 type-code",
};

const lines: Record<string, string> = {
  FencedCode: "bg-muted type-code",
  CodeBlock: "bg-muted type-code",
  Blockquote: "border-l-2 pl-4 text-muted-foreground",
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

  const lineClass = headings[name] ?? lines[name];
  if (lineClass) {
    const last = doc.lineAt(node.to).number;
    for (let line = doc.lineAt(node.from).number; line <= last; line++) {
      found.push(
        Decoration.line({ class: lineClass }).range(doc.line(line).from),
      );
    }
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
    case "CodeMark":
      if (node.node.parent?.name === "InlineCode") {
        hide(node.from, node.to);
      }
      break;
    case "ListMark":
      if (node.node.parent?.parent?.name === "BulletList") {
        found.push(
          Decoration.replace({ widget: new Bullet() }).range(
            node.from,
            node.to,
          ),
        );
      }
      break;
    case "TaskMarker":
      found.push(
        Decoration.replace({
          widget: new Checkbox(text(node.from, node.to) !== "[ ]", node.from),
        }).range(node.from, node.to),
      );
      break;
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
  const active = activeLines(view);
  const { doc } = view.state;

  for (const { from, to } of view.visibleRanges) {
    syntaxTree(view.state).iterate({
      from,
      to,
      enter: (node) => {
        const first = doc.lineAt(node.from).number;
        const last = doc.lineAt(node.to).number;
        let editing = false;
        for (let line = first; line <= last && !editing; line++) {
          editing = active.has(line);
        }

        decorate(view.state, node, editing, found);
      },
    });
  }

  return Decoration.set(found, true);
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
