import { type ReactNode, type RefObject, useEffect, useRef } from "react";

import { Layer } from "@/components/frame";
import { PageClose } from "@/lib/page-close";

interface Props {
  title: string;
  onClose: () => void;
  children: ReactNode;
}

export function PageFrame({ title, onClose, children }: Props) {
  const frame = useRef<HTMLElement>(null);

  useEscape(frame, onClose);

  return (
    <Layer ref={frame} aria-label={title} className="relative h-full">
      <h1 className="sr-only">{title}</h1>
      <PageClose value={onClose}>{children}</PageClose>
    </Layer>
  );
}

export function FullPage({
  title,
  onClose,
  children,
}: {
  title: string;
  onClose?: () => void;
  children: ReactNode;
}) {
  const frame = useRef<HTMLElement>(null);

  useEscape(frame, onClose);

  return (
    <section
      ref={frame}
      aria-label={title}
      className="relative flex min-h-0 flex-1 flex-col"
    >
      <h1 className="sr-only">{title}</h1>
      {onClose ? <PageClose value={onClose}>{children}</PageClose> : children}
    </section>
  );
}

function useEscape(
  frame: RefObject<HTMLElement | null>,
  onClose: (() => void) | undefined,
) {
  useEffect(() => {
    if (!onClose) {
      return;
    }

    const down = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || event.defaultPrevented) {
        return;
      }

      const target = event.target;
      const inside =
        target === document.body ||
        (target instanceof Node && frame.current?.contains(target) === true);
      if (inside && !editable(target)) {
        onClose();
      }
    };

    window.addEventListener("keydown", down);

    return () => {
      window.removeEventListener("keydown", down);
    };
  }, [frame, onClose]);
}

function editable(target: EventTarget | null) {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable ||
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement)
  );
}
