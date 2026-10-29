import { openUrl } from "@tauri-apps/plugin-opener";
import DOMPurify from "dompurify";
import { type MouseEvent, useMemo } from "react";

DOMPurify.addHook("afterSanitizeAttributes", (node) => {
  if (node.tagName === "INPUT") {
    node.setAttribute("type", "checkbox");
    node.setAttribute("disabled", "");
  }
});

interface Props {
  html: string;
  empty?: string;
}

export function RenderedHtml({ html, empty }: Props) {
  const clean = useMemo(
    () =>
      DOMPurify.sanitize(html, {
        USE_PROFILES: { html: true },
        FORBID_TAGS: ["style", "form", "button"],
        FORBID_ATTR: ["style"],
      }),
    [html],
  );

  if (!clean.trim()) {
    return empty ? (
      <p className="type-body text-muted-foreground">{empty}</p>
    ) : null;
  }

  return (
    <div
      className="rendered min-w-0 type-body"
      onClick={follow}
      onAuxClick={(event) => {
        if (event.button === 1) {
          follow(event);
        }
      }}
      dangerouslySetInnerHTML={{ __html: clean }}
    />
  );
}

function follow(event: MouseEvent) {
  const link =
    event.target instanceof Element ? event.target.closest("a") : null;
  if (!link) {
    return;
  }

  event.preventDefault();
  const href = external(link.getAttribute("href"));
  if (href) {
    void openUrl(href);
  }
}

function external(href: string | null) {
  if (!href) {
    return undefined;
  }

  try {
    const url = new URL(href);

    return url.protocol === "https:" || url.protocol === "http:"
      ? url.href
      : undefined;
  } catch {
    return undefined;
  }
}
