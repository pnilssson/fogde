import { CheckIcon } from "@phosphor-icons/react";

export function CopiedNotice({ copied }: { copied: boolean }) {
  return (
    <div
      role="status"
      className="pointer-events-none absolute bottom-4 left-1/2 z-10 -translate-x-1/2"
    >
      {copied && (
        <div className="flex animate-in items-center gap-2 rounded-lg bg-popover px-4 py-2 type-body text-popover-foreground shadow-surface-5 duration-150 ease-out fade-in-0">
          <CheckIcon aria-hidden />
          Copied
        </div>
      )}
    </div>
  );
}
