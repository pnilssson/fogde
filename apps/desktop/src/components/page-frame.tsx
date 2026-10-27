import { XIcon } from "@phosphor-icons/react";
import { type ReactNode, useEffect, useRef } from "react";

import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { Machine } from "@/lib/machines";

interface Props {
  title: string;
  machine: Machine;
  machines: Machine[];
  onMachine: (machine: string) => void;
  onClose: () => void;
  leading?: ReactNode;
  actions?: ReactNode;
  children: ReactNode;
}

export function PageFrame({
  title,
  machine,
  machines,
  onMachine,
  onClose,
  leading,
  actions,
  children,
}: Props) {
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
      className="flex h-full min-h-0 flex-col"
    >
      <header className="flex h-8 shrink-0 items-center gap-2 border-b px-2">
        {leading}
        <h1 className="truncate type-caption">{title}</h1>
        {machines.length > 1 && (
          <Select
            items={machines.map(({ id, name }) => ({ label: name, value: id }))}
            value={machine.id}
            onValueChange={(next) => {
              if (next) {
                onMachine(next);
              }
            }}
          >
            <SelectTrigger size="sm" aria-label="Machine">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {machines.map(({ id, name }) => (
                <SelectItem key={id} value={id}>
                  {name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
        <span className="flex-1" />
        {actions}
        <Tooltip>
          <TooltipTrigger
            render={
              <Button
                variant="ghost"
                size="icon-xs"
                aria-label={`Close ${title}`}
                onClick={onClose}
              />
            }
          >
            <XIcon />
          </TooltipTrigger>
          <TooltipContent>Close</TooltipContent>
        </Tooltip>
      </header>
      <div className="relative min-h-0 flex-1">{children}</div>
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
