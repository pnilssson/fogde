import { type ReactNode, type RefObject, useRef } from "react";

import { Layer } from "@/components/frame";
import { useWindowKey } from "@/hooks/use-window-key";
import { editable } from "@/lib/dialogs";
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
  useWindowKey((event) => {
    const target = event.target;
    const inside =
      target === document.body ||
      (target instanceof Node && frame.current?.contains(target) === true);

    return event.key === "Escape" && inside && !editable(target)
      ? onClose
      : undefined;
  }, false);
}
