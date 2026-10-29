import {
  type ReactNode,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";

import { Layer } from "@/components/frame";
import { SidebarMenu, useSidebar } from "@/components/ui/sidebar";
import { UpdateItem } from "@/components/update-item";
import { MachineGroup } from "@/features/workspaces/components/machine-group";
import type { useGroups } from "@/features/workspaces/hooks/use-terminals";
import { share } from "@/features/workspaces/lib/heights";
import type { Selection, View } from "@/features/workspaces/lib/selection";

const gap = 8;

interface Props {
  groups: ReturnType<typeof useGroups>;
  selected: View | undefined;
  onSelect: (view: View, terminal?: Selection) => void;
  onStart: (view: View, folder: string) => void;
  meters: ReactNode;
}

export function WorkspaceSidebar({
  groups,
  selected,
  onSelect,
  onStart,
  meters,
}: Props) {
  const { setOpenMobile } = useSidebar();
  const stack = useRef<HTMLDivElement>(null);
  const [room, setRoom] = useState(0);
  const [natural, setNatural] = useState<Partial<Record<string, number>>>({});
  const measure = useCallback((machine: string, height: number) => {
    setNatural((known) =>
      known[machine] === height ? known : { ...known, [machine]: height },
    );
  }, []);

  useEffect(() => {
    const element = stack.current;
    if (!element) {
      return;
    }

    const observer = new ResizeObserver(() => {
      setRoom(element.clientHeight);
    });
    observer.observe(element);

    return () => {
      observer.disconnect();
    };
  }, []);

  const sizes = groups.map(({ machine }) => natural[machine.id]);
  const heights =
    room > 0 && sizes.every((size): size is number => size !== undefined)
      ? share(room - gap * (groups.length - 1), sizes)
      : undefined;

  return (
    <>
      <div ref={stack} className="flex min-h-0 flex-1 flex-col gap-2">
        {groups.map(({ machine, workspaces }, index) => (
          <MachineGroup
            key={machine.id}
            machine={machine}
            workspaces={workspaces}
            selected={selected}
            height={heights?.[index]}
            onMeasure={measure}
            onSelect={(workspace, terminal) => {
              onSelect(workspace, terminal);
              setOpenMobile(false);
            }}
            onStart={(workspace, folder) => {
              onStart(workspace, folder);
              setOpenMobile(false);
            }}
          />
        ))}
      </div>
      <Layer className="shrink-0 p-2 [&:not(:has(button,li))]:hidden">
        {meters}
        <SidebarMenu>
          <UpdateItem />
        </SidebarMenu>
      </Layer>
    </>
  );
}
