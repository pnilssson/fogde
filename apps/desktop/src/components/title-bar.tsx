import { MinusIcon, SquareIcon, XIcon } from "@phosphor-icons/react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { ReactNode } from "react";
import { cn } from "cn";

import type { Platform } from "@/generated/protocol";
import { frame } from "@/lib/frame";

const current = getCurrentWindow();

const chrome: Record<
  Platform,
  { inset: string; controls: boolean; bare: boolean }
> = {
  windows: { inset: "", controls: true, bare: false },
  macos: { inset: "pl-20", controls: false, bare: false },
  linux: { inset: "", controls: false, bare: true },
};

const { inset, controls, bare } = chrome[frame.platform];

interface Props {
  start?: ReactNode;
  center?: ReactNode;
}

export function TitleBar({ start, center }: Props) {
  if (bare && !start && !center) {
    return null;
  }

  return (
    <header
      className={cn(
        "grid h-10 shrink-0 grid-cols-[1fr_auto_1fr] items-center",
        inset,
      )}
    >
      <div className="flex h-full min-w-0 items-center">
        {start && <div className="flex items-center gap-2 px-2">{start}</div>}
        <div data-tauri-drag-region className="h-full flex-1" />
      </div>
      <div className="flex items-center">{center}</div>
      <div className="flex h-full min-w-0 items-center">
        <div data-tauri-drag-region className="h-full flex-1" />
        {controls && (
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
      </div>
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
