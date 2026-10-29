import { cn } from "cn";
import type { ReactNode } from "react";

import { Layer } from "@/components/frame";
import { Sidebar, SidebarInset, useSidebar } from "@/components/ui/sidebar";

interface Props {
  side: ReactNode;
  away?: boolean;
  children: ReactNode;
}

export function Shell({ side, away = false, children }: Props) {
  const { isMobile } = useSidebar();

  return (
    <div
      inert={away}
      className={cn("relative flex min-h-0 flex-1", away && "opacity-0")}
    >
      {!(away && isMobile) && (
        <Sidebar className="absolute h-full group-data-[side=left]:border-r-0">
          <div className="flex min-h-0 flex-1 flex-col gap-2 pb-2 pl-2">
            {side}
          </div>
        </Sidebar>
      )}
      <SidebarInset className="min-h-0 min-w-0 overflow-hidden bg-transparent px-2 pb-2">
        {children}
      </SidebarInset>
    </div>
  );
}

export function PageShell({
  side = <Layer className="flex-1" />,
  children,
}: {
  side?: ReactNode;
  children: ReactNode;
}) {
  return (
    <Shell side={side}>
      <Layer className="relative flex-1">{children}</Layer>
    </Shell>
  );
}
