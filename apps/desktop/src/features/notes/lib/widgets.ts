import { type EditorView, WidgetType } from "@codemirror/view";

export class Bullet extends WidgetType {
  override eq() {
    return true;
  }

  toDOM() {
    const bullet = document.createElement("span");
    bullet.textContent = "•";
    bullet.className = "text-muted-foreground";

    return bullet;
  }
}

export class Rule extends WidgetType {
  override eq() {
    return true;
  }

  toDOM() {
    const rule = document.createElement("span");
    rule.className = "inline-block w-full border-t align-middle";

    return rule;
  }
}

export class Checkbox extends WidgetType {
  private readonly checked: boolean;
  private readonly at: number;

  constructor(checked: boolean, at: number) {
    super();
    this.checked = checked;
    this.at = at;
  }

  override eq(other: Checkbox) {
    return other.checked === this.checked && other.at === this.at;
  }

  toDOM(view: EditorView) {
    const box = document.createElement("input");
    box.type = "checkbox";
    box.checked = this.checked;
    box.className = "mr-1 align-middle";
    box.setAttribute("aria-label", this.checked ? "Done" : "Not done");
    box.addEventListener("mousedown", (event) => {
      event.preventDefault();
    });
    box.addEventListener("click", (event) => {
      event.preventDefault();
      view.dispatch({
        changes: {
          from: this.at + 1,
          to: this.at + 2,
          insert: this.checked ? " " : "x",
        },
      });
    });

    return box;
  }

  override ignoreEvent() {
    return true;
  }
}
