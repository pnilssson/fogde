import { syntaxTree } from "@codemirror/language";
import {
  type EditorState,
  type Range,
  StateEffect,
  StateField,
  type Text,
} from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  EditorView,
  WidgetType,
} from "@codemirror/view";
import type { SyntaxNode } from "@lezer/common";

import { append, type Inline, inline } from "@/features/notes/lib/inline";

type Align = "text-left" | "text-center" | "text-right";

function cells(doc: Text, row: SyntaxNode) {
  const pipes = row.getChildren("TableDelimiter");
  const edges = [row.from, ...pipes.flatMap((pipe) => [pipe.from, pipe.to])];
  edges.push(row.to);

  const found: Inline[][] = [];
  for (let index = 0; index < edges.length; index += 2) {
    const from = edges[index] ?? row.from;
    const to = edges[index + 1] ?? row.to;
    const cell = row
      .getChildren("TableCell")
      .find((node) => node.from >= from && node.to <= to);
    found.push(cell ? inline(doc, cell, cell.from, cell.to) : []);
  }

  if (pipes[0]?.from === row.from) {
    found.shift();
  }
  if (pipes.at(-1)?.to === row.to) {
    found.pop();
  }

  return found;
}

function alignments(text: string) {
  const parts = text.split("|").map((part) => part.trim());
  if (parts[0] === "") {
    parts.shift();
  }
  if (parts.at(-1) === "") {
    parts.pop();
  }

  return parts.map((part): Align => {
    if (part.endsWith(":")) {
      return part.startsWith(":") ? "text-center" : "text-right";
    }

    return "text-left";
  });
}

class Table extends WidgetType {
  private readonly source: string;
  private readonly head: Inline[][];
  private readonly rows: Inline[][][];
  private readonly align: Align[];

  constructor(doc: Text, node: SyntaxNode) {
    super();
    this.source = doc.sliceString(node.from, node.to);
    const header = node.getChild("TableHeader");
    this.head = header ? cells(doc, header) : [];
    const columns = this.head.length;
    this.rows = node.getChildren("TableRow").map((row) => {
      const found = cells(doc, row).slice(0, columns);
      while (found.length < columns) {
        found.push([]);
      }

      return found;
    });
    const delimiter = node.getChild("TableDelimiter");
    this.align = delimiter
      ? alignments(doc.sliceString(delimiter.from, delimiter.to))
      : [];
  }

  override eq(other: Table) {
    return other.source === this.source;
  }

  toDOM() {
    const wrapper = document.createElement("div");
    wrapper.className = "overflow-x-auto";
    const table = document.createElement("table");
    table.className = "border-collapse";
    const row = (values: Inline[][], tag: "th" | "td") => {
      const element = document.createElement("tr");
      values.forEach((value, index) => {
        const cell = document.createElement(tag);
        const align = this.align[index] ?? "text-left";
        cell.className =
          tag === "th"
            ? `border px-4 py-2 align-top font-semibold ${align}`
            : `border px-4 py-2 align-top ${align}`;
        append(cell, value);
        element.append(cell);
      });

      return element;
    };

    const head = document.createElement("thead");
    head.append(row(this.head, "th"));
    const body = document.createElement("tbody");
    for (const values of this.rows) {
      body.append(row(values, "td"));
    }
    table.append(head, body);
    wrapper.append(table);

    return wrapper;
  }

  override ignoreEvent() {
    return false;
  }
}

const focus = StateEffect.define<boolean>();

const focused = StateField.define<boolean>({
  create: () => false,
  update: (value, transaction) => {
    for (const effect of transaction.effects) {
      if (effect.is(focus)) {
        value = effect.value;
      }
    }

    return value;
  },
  provide: () =>
    EditorView.focusChangeEffect.of((_, focusing) => focus.of(focusing)),
});

function build(state: EditorState) {
  const found: Range<Decoration>[] = [];
  const { doc, selection } = state;
  const editing = state.field(focused);

  for (
    let node = syntaxTree(state).topNode.firstChild;
    node;
    node = node.nextSibling
  ) {
    if (node.name !== "Table") {
      continue;
    }

    const { from, to } = node;
    const touched =
      editing &&
      selection.ranges.some((range) => range.from <= to && range.to >= from);
    const whole = doc.lineAt(from).from === from && doc.lineAt(to).to === to;
    if (!touched && whole) {
      found.push(
        Decoration.replace({
          widget: new Table(doc, node),
          block: true,
        }).range(from, to),
      );
    }
  }

  return Decoration.set(found);
}

const field = StateField.define<DecorationSet>({
  create: build,
  update: (decorations, transaction) =>
    transaction.docChanged ||
    transaction.selection ||
    transaction.startState.field(focused) !==
      transaction.state.field(focused) ||
    syntaxTree(transaction.startState) !== syntaxTree(transaction.state)
      ? build(transaction.state)
      : decorations,
  provide: (decorations) => EditorView.decorations.from(decorations),
});

export const tables = [focused, field];
