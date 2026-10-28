import { type ReactNode, useEffect, useRef } from "react";

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
    <section
      ref={frame}
      aria-label={title}
      className="relative h-full min-h-0 overflow-hidden rounded-xl bg-surface-3 shadow-surface-2"
    >
      <h1 className="sr-only">{title}</h1>
      {children}
    </section>
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
