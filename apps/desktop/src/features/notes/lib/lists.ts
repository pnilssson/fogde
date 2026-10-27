import type { EditorState, Range } from "@codemirror/state";
import { Decoration } from "@codemirror/view";
import type { SyntaxNode } from "@lezer/common";

import { Bullet, Checkbox } from "@/features/notes/lib/widgets";

const ordered = "cm-marker pr-2 text-right text-muted-foreground";

export type Items = Map<
  number,
  { depth: number; first: boolean; column: number }
>;

function spaced(state: EditorState, at: number) {
  const line = state.doc.lineAt(at);
  const space = /^[ \t]*/.exec(line.text.slice(at - line.from))?.[0].length;

  return at + (space ?? 0);
}

export function listItem(
  state: EditorState,
  node: SyntaxNode,
  active: Set<number>,
  items: Items,
  found: Range<Decoration>[],
) {
  const { doc } = state;
  const mark = node.getChild("ListMark");
  if (!mark) {
    return;
  }

  let depth = 0;
  for (let parent = node.parent; parent; parent = parent.parent) {
    if (parent.name === "BulletList" || parent.name === "OrderedList") {
      depth++;
    }
  }

  const end = spaced(state, mark.to);
  const start = doc.lineAt(mark.from);
  const last = doc.lineAt(node.to).number;
  const column = end - start.from;
  for (let line = start.number; line <= last; line++) {
    items.set(line, { depth, first: line === start.number, column });
  }

  const bullet = node.parent?.name === "BulletList";
  const task = node.getChild("Task")?.getChild("TaskMarker");
  if (active.has(start.number)) {
    found.push(
      Decoration.mark({
        class: bullet ? "cm-marker text-center" : ordered,
      }).range(mark.from, end),
    );
  } else if (task) {
    found.push(
      Decoration.replace({
        widget: new Checkbox(
          doc.sliceString(task.from, task.to) !== "[ ]",
          task.from,
        ),
      }).range(mark.from, spaced(state, task.to)),
    );
  } else if (bullet) {
    found.push(
      Decoration.replace({ widget: new Bullet() }).range(mark.from, end),
    );
  } else {
    found.push(Decoration.mark({ class: ordered }).range(mark.from, end));
  }
}

export function indent(state: EditorState, items: Items) {
  const found: Range<Decoration>[] = [];
  for (const [number, { depth, first, column }] of items) {
    const line = state.doc.line(number);
    found.push(
      Decoration.line({
        class: first ? "cm-item" : "cm-item-rest",
        attributes: { style: `--depth: ${String(depth)}` },
      }).range(line.from),
    );

    const leading = /^[ \t]*/.exec(line.text)?.[0].length ?? 0;
    const space = first ? leading : Math.min(leading, column);
    if (space > 0) {
      found.push(
        Decoration.mark({ class: "cm-indent" }).range(
          line.from,
          line.from + space,
        ),
      );
    }
  }

  return found;
}
