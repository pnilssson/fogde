import { MinusIcon, SquareIcon, XIcon } from "@phosphor-icons/react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { ReactNode } from "react";
import { cn } from "cn";

import { frame } from "@/lib/frame";

const current = getCurrentWindow();

export function TitleBar({ children }: { children?: ReactNode }) {
  if (frame.platform === "linux" && !children) {
    return null;
  }

  return (
    <header
      className={cn(
        "flex h-10 shrink-0 items-center",
        frame.platform === "macos" && "pl-20",
      )}
    >
      {children && (
        <div className="flex items-center gap-2 px-2">{children}</div>
      )}
      <div data-tauri-drag-region className="h-full flex-1" />
      {frame.platform === "windows" && (
        <div className="flex h-full shrink-0">
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
        </div>
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
        "flex w-12 items-center justify-center hover:bg-hover focus-visible:bg-hover focus-visible:outline-none",
        className,
      )}
    >
      {children}
    </button>
  );
}
