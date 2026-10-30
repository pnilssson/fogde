import { cn } from "cn";
import type { ComponentProps, ReactNode } from "react";

import { ScrollArea } from "@/components/ui/scroll-area";

export function Bar({ className, ...props }: ComponentProps<"header">) {
  return (
    <header
      {...props}
      className={cn("flex h-12 shrink-0 items-center gap-2 px-2", className)}
    />
  );
}

export function Layer({ className, ...props }: ComponentProps<"section">) {
  return (
    <section
      {...props}
      className={cn(
        "flex min-h-0 flex-col overflow-hidden rounded-xl bg-surface-3 shadow-surface-2",
        className,
      )}
    />
  );
}

export function SideScroll({ children }: { children: ReactNode }) {
  return (
    <ScrollArea className="group/side min-h-0 flex-1">
      <div className="px-2 pb-2 group-data-has-overflow-y/side:pr-4">
        {children}
      </div>
    </ScrollArea>
  );
}

export function Body({ children }: { children: ReactNode }) {
  return <div className="flex min-h-0 flex-1 gap-2 px-2 pb-2">{children}</div>;
}

export function Main({ children }: { children: ReactNode }) {
  return (
    <div className="relative flex min-w-0 flex-1 flex-col gap-2">
      {children}
    </div>
  );
}

export function Scroll({
  className,
  children,
}: {
  className?: string;
  children: ReactNode;
}) {
  return (
    <ScrollArea className="min-h-0 flex-1">
      <div className={cn("pr-4", className)}>{children}</div>
    </ScrollArea>
  );
}
