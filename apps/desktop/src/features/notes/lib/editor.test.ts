// @vitest-environment happy-dom
import { EditorView } from "@codemirror/view";
import { GFM, parser } from "@lezer/markdown";
import { afterEach, expect, test, vi } from "vitest";

import { noteState, replaceText } from "@/features/notes/lib/editor";
import { wikilinkParts, wikilinks } from "@/features/notes/lib/wikilinks";

const sample = [
  "# Plan",
  "",
  "See [[login]], [[app/plan#Next|the plan]] and ![[diagram.png]].",
  "Code `[[not a link]]`, an [[unclosed link and a [web link](https://example.com).",
  "",
  "- [ ] ship *it* **today** ~~maybe~~",
  "- [x] done",
  "",
  "> quoted",
  "",
  "| App | State |",
  "|-----|-------|",
  "| web | done |",
  "",
  "```ts",
  "const a = 1;",
  "```",
  "",
  "---",
  "",
].join("\n");

afterEach(() => {
  document.body.replaceChildren();
});

test("parses wikilinks but not embeds, code or unclosed brackets", () => {
  const tree = parser.configure([GFM, wikilinks]).parse(sample);
  const found: string[] = [];
  tree.iterate({
    enter: (node) => {
      if (node.name === "WikiLink") {
        found.push(sample.slice(node.from, node.to));
      }
    },
  });

  expect(found).toEqual(["[[login]]", "[[app/plan#Next|the plan]]"]);
  expect(wikilinkParts("app/plan#Next|the plan")).toEqual({
    target: "app/plan#Next",
    label: "the plan",
  });
});

test("renders a note with links that open and leaves its text untouched", () => {
  const wikilink = vi.fn();
  const view = new EditorView({
    parent: document.body,
    state: noteState(
      sample.replaceAll("\n", "\r\n"),
      { wikilink, link: vi.fn() },
      vi.fn(),
    ),
  });

  const rendered = view.contentDOM.querySelector("[data-wikilink]");
  rendered?.dispatchEvent(
    new MouseEvent("mousedown", { bubbles: true, button: 0 }),
  );

  expect(rendered?.textContent).toBe("login");
  expect(wikilink).toHaveBeenCalledWith("login");
  expect(
    view.contentDOM.querySelectorAll('input[type="checkbox"]'),
  ).toHaveLength(2);
  expect(view.state.sliceDoc()).toBe(sample.replaceAll("\n", "\r\n"));
  view.destroy();
});

test("renders a table as a grid until the cursor enters it", () => {
  const wikilink = vi.fn();
  const text = [
    "| App | State | Code \\| note |",
    "|:----|------:|:-------------:|",
    "| web | **done** | [[login]] |",
    "| | | |",
    "| api |",
    "",
  ].join("\n");
  const view = new EditorView({
    parent: document.body,
    state: noteState(text, { wikilink, link: vi.fn() }, vi.fn()),
  });
  const grid = () =>
    [...view.dom.querySelectorAll("tr")].map((row) =>
      [...row.children].map((cell) => cell.textContent),
    );

  expect(grid()).toEqual([
    ["App", "State", "Code | note"],
    ["web", "done", "login"],
    ["", "", ""],
    ["api", "", ""],
  ]);
  expect(view.dom.querySelector("th:nth-child(2)")?.className).toContain(
    "text-right",
  );
  expect(view.dom.querySelector("th:nth-child(3)")?.className).toContain(
    "text-center",
  );

  view.dom
    .querySelector("td [data-wikilink]")
    ?.dispatchEvent(new MouseEvent("mousedown", { bubbles: true, button: 0 }));

  expect(wikilink).toHaveBeenCalledWith("login");

  const focus = (focusing: boolean) => {
    view.dispatch({
      effects: view.state
        .facet(EditorView.focusChangeEffect)
        .flatMap((effect) => effect(view.state, focusing) ?? []),
    });
  };
  focus(true);
  view.dispatch({ selection: { anchor: text.indexOf("web") } });

  expect(view.dom.querySelector("table")).toBeNull();

  focus(false);

  expect(view.dom.querySelector("table")).not.toBeNull();
  expect(view.state.sliceDoc()).toBe(text);
  view.destroy();
});

test("a note replaced from disk keeps the line endings it has on disk", () => {
  const edited = vi.fn();
  const view = new EditorView({
    parent: document.body,
    state: noteState("", { wikilink: vi.fn(), link: vi.fn() }, edited),
  });

  replaceText(view, "# Plan\r\n\r\n- login\r\n");

  expect(view.state.sliceDoc()).toBe("# Plan\r\n\r\n- login\r\n");

  replaceText(view, "# Plan\r\n\r\n- login\r\n- signup\r\n- reset\r\n");

  expect(view.state.sliceDoc()).toBe(
    "# Plan\r\n\r\n- login\r\n- signup\r\n- reset\r\n",
  );

  replaceText(view, "# Plan\n\n- signup\n");

  expect(view.state.sliceDoc()).toBe("# Plan\n\n- signup\n");
  expect(edited).not.toHaveBeenCalled();
  view.destroy();
});
