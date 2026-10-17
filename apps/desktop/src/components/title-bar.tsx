import { MinusIcon, SquareIcon, XIcon } from "@phosphor-icons/react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { ReactNode } from "react";
import { cn } from "cn";

import { frame } from "@/lib/frame";

const current = getCurrentWindow();

export function TitleBar() {
  if (frame.platform === "linux") {
    return null;
  }

  return (
    <header data-tauri-drag-region className="flex h-10 shrink-0 justify-end">
      {frame.platform === "windows" && (
        <>
          <Control label="Minimize" onClick={() => void current.minimize()}>
            <MinusIcon />
          </Control>
          <Control
            label="Maximize"
            onClick={() => void current.toggleMaximize()}
          >
            <SquareIcon />
          </Control>
          <Control
            label="Close"
            onClick={() => void current.close()}
            className="hover:bg-destructive hover:text-white"
          >
            <XIcon />
          </Control>
        </>
      )}
    </header>
  );
}

interface ControlProps {
  label: string;
  onClick: () => void;
  className?: string;
  children: ReactNode;
}

function Control({ label, onClick, className, children }: ControlProps) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={onClick}
      className={cn(
        "flex w-12 items-center justify-center hover:bg-foreground/10 focus-visible:bg-foreground/10 focus-visible:outline-none",
        className,
      )}
    >
      {children}
    </button>
  );
}
