import {
  defaultKeymap,
  history,
  historyKeymap,
  indentWithTab,
} from "@codemirror/commands";
import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import {
  Annotation,
  Compartment,
  EditorSelection,
  EditorState,
  type Extension,
  Text,
} from "@codemirror/state";
import {
  type Command,
  drawSelection,
  EditorView,
  keymap,
} from "@codemirror/view";
import { tags } from "@lezer/highlight";

import { diff } from "@/features/notes/lib/merge";
import { preview } from "@/features/notes/lib/preview";
import { tables } from "@/features/notes/lib/tables";
import { wikilinks } from "@/features/notes/lib/wikilinks";

const external = Annotation.define<boolean>();

const endings = new Compartment();

export interface Links {
  wikilink: (target: string) => void;
  link: (href: string) => void;
}

const muted = HighlightStyle.define([
  { tag: tags.processingInstruction, class: "text-muted-foreground" },
  { tag: tags.url, class: "text-muted-foreground" },
  { tag: tags.contentSeparator, class: "text-muted-foreground" },
]);

const theme = EditorView.theme({
  "&": { flexGrow: 1, backgroundColor: "transparent" },
  "&.cm-focused": { outline: "none" },
  ".cm-scroller": {
    flexGrow: 1,
    overflow: "visible",
    fontFamily: "inherit",
    lineHeight: "inherit",
  },
  ".cm-content": {
    maxWidth: "calc(65ch + 64px)",
    margin: "0 auto",
    padding: "24px 32px",
    caretColor: "var(--foreground)",
  },
  ".cm-cursor": { borderLeftColor: "var(--foreground)" },
  ".cm-selectionBackground, &.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground":
    { background: "var(--selected)" },
  ".cm-line": { padding: "0" },
  ".cm-h1, .cm-h2": { paddingTop: "32px" },
  ".cm-h3, .cm-h4, .cm-h5, .cm-h6": { paddingTop: "16px" },
  ".cm-content > .cm-line:first-child": { paddingTop: "0" },
  ".cm-code": { paddingInline: "16px" },
  ".cm-quote": { paddingLeft: "16px" },
  ".cm-item": {
    paddingLeft: "calc(var(--depth) * 32px)",
    textIndent: "calc(var(--depth) * -32px)",
  },
  ".cm-item-rest": { paddingLeft: "calc(var(--depth) * 32px)" },
  ".cm-indent": {
    display: "inline-block",
    width: "calc((var(--depth) - 1) * 32px)",
    textIndent: "0",
  },
  ".cm-item-rest .cm-indent": { width: "0" },
  ".cm-item-rest.cm-code": {
    marginLeft: "calc(var(--depth) * 32px)",
    paddingLeft: "16px",
  },
  ".cm-marker": { display: "inline-block", minWidth: "32px", textIndent: "0" },
});

function wrap(marker: string): Command {
  return (view) => {
    view.dispatch(
      view.state.changeByRange((range) => {
        const { from, to } = range;
        const before = view.state.sliceDoc(from - marker.length, from);
        const after = view.state.sliceDoc(to, to + marker.length);
        if (before === marker && after === marker) {
          return {
            changes: [
              { from: from - marker.length, to: from },
              { from: to, to: to + marker.length },
            ],
            range: EditorSelection.range(
              from - marker.length,
              to - marker.length,
            ),
          };
        }

        return {
          changes: [
            { from, insert: marker },
            { from: to, insert: marker },
          ],
          range: EditorSelection.range(
            from + marker.length,
            to + marker.length,
          ),
        };
      }),
    );

    return true;
  };
}

function follow(links: Links) {
  return EditorView.domEventHandlers({
    mousedown: (event) => {
      if (event.button !== 0 || !(event.target instanceof Element)) {
        return false;
      }

      const target = event.target.closest("[data-link], [data-wikilink]");
      const wikilink = target?.getAttribute("data-wikilink");
      const href = target?.getAttribute("data-link");
      if (wikilink) {
        links.wikilink(wikilink);
      } else if (href) {
        links.link(href);
      } else {
        return false;
      }

      event.preventDefault();
      return true;
    },
  });
}

function lineEndings(text: string): Extension {
  return text.includes("\r\n") ? EditorState.lineSeparator.of("\r\n") : [];
}

export function replaceText(view: EditorView, text: string) {
  if (text.includes("\r\n") !== (view.state.lineBreak === "\r\n")) {
    view.dispatch({ effects: endings.reconfigure(lineEndings(text)) });
  }

  const change = diff(
    view.state.doc.toString(),
    view.state.toText(text).toString(),
  );
  if (change) {
    view.dispatch({
      changes: { ...change, insert: Text.of(change.insert.split("\n")) },
      annotations: external.of(true),
    });
  }
}

export function noteState(
  text: string,
  links: Links,
  edited: () => void,
): EditorState {
  const extensions: Extension[] = [
    endings.of(lineEndings(text)),
    history(),
    drawSelection(),
    EditorView.lineWrapping,
    EditorView.contentAttributes.of({ class: "type-note" }),
    markdown({ base: markdownLanguage, extensions: [wikilinks] }),
    syntaxHighlighting(muted),
    preview,
    tables,
    theme,
    follow(links),
    keymap.of([
      { key: "Mod-b", run: wrap("**") },
      { key: "Mod-i", run: wrap("*") },
      indentWithTab,
      ...historyKeymap,
      ...defaultKeymap,
    ]),
    EditorView.updateListener.of((update) => {
      if (
        update.docChanged &&
        !update.transactions.some((transaction) =>
          transaction.annotation(external),
        )
      ) {
        edited();
      }
    }),
  ];

  return EditorState.create({ doc: text, extensions });
}
