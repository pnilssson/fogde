import { type ReactNode, useEffect, useRef } from "react";

import { Layer } from "@/components/frame";
import { PageClose } from "@/lib/page-close";

interface Props {
  title: string;
  onClose: () => void;
  children: ReactNode;
}

export function PageFrame({ title, onClose, children }: Props) {
  const frame = useRef<HTMLElement>(null);

  useEffect(() => {
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
  }, [onClose]);

  return (
    <Layer ref={frame} aria-label={title} className="relative h-full">
      <h1 className="sr-only">{title}</h1>
      <PageClose value={onClose}>{children}</PageClose>
    </Layer>
  );
}

function editable(target: EventTarget | null) {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable ||
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement)
  );
}

export function FullPage({
  title,
  children,
}: {
  title: string;
  children: ReactNode;
}) {
  return (
    <section
      aria-label={title}
      className="relative flex min-h-0 flex-1 flex-col"
    >
      <h1 className="sr-only">{title}</h1>
      {children}
    </section>
  );
}
