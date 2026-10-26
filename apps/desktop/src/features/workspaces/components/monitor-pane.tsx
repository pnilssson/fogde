import { PulseIcon } from "@phosphor-icons/react";
import type { IDockviewPanelProps } from "dockview-react";
import { useContext } from "react";

import { TileBar } from "@/features/workspaces/components/tile-bar";
import { useCloseTile } from "@/features/workspaces/hooks/use-workspaces";
import { useTiles } from "@/features/workspaces/lib/layouts";
import {
  type Selection,
  WorkspaceContext,
} from "@/features/workspaces/lib/selection";
import { MonitorSlot } from "@/features/workspaces/lib/slot";
import { activate } from "@/features/workspaces/lib/tiles";

export function MonitorPane({ api, params }: IDockviewPanelProps<Selection>) {
  const render = useContext(MonitorSlot);
  const { groups } = useContext(WorkspaceContext);
  const active = useTiles((tiles) => tiles.focused === api.id);
  const closing = useCloseTile();
  const group = groups.find(({ machine }) => machine.id === params.machine);
  const title = group ? `Monitor on ${group.machine.name}` : "Monitor";

  return (
    <section
      aria-label={title}
      className="flex h-full min-h-0 flex-col"
      onPointerDown={() => {
        activate(api);
      }}
    >
      <TileBar
        item={{ kind: "monitor", ...params }}
        active={active}
        icon={<PulseIcon className="size-4 shrink-0" />}
        title={title}
        onClose={() => {
          if (group) {
            closing.mutate({ machine: group.machine, tile: params.id });
          } else {
            api.close();
          }
        }}
      />
      <div className="min-h-0 flex-1">
        {group &&
          render({
            machine: group.machine,
            workspaces: group.workspaces.data ?? [],
          })}
      </div>
    </section>
  );
}
