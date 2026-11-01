import { type ReactNode, useContext } from "react";
import { createPortal } from "react-dom";

import { Layer } from "@/components/frame";
import { Sidebar, SidebarInset } from "@/components/ui/sidebar";
import { SideSlot } from "@/lib/side-slot";

interface Props {
  side: ReactNode;
  children: ReactNode;
}

export function Shell({ side, children }: Props) {
  return (
    <div className="relative flex min-h-0 flex-1">
      <Sidebar className="absolute h-full group-data-[side=left]:border-r-0">
        <div className="flex min-h-0 flex-1 flex-col gap-2 pb-2 pl-2">
          {side}
        </div>
      </Sidebar>
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
  const slot = useContext(SideSlot);

  return (
    <>
      {slot && createPortal(side, slot)}
      <Layer className="relative flex-1">{children}</Layer>
    </>
  );
}
