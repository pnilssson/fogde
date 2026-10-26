import { PulseIcon } from "@phosphor-icons/react";

import { TileItem } from "@/features/workspaces/components/tile-item";
import { useCloseTile } from "@/features/workspaces/hooks/use-workspaces";
import type { MonitorTile } from "@/features/workspaces/lib/tile-kinds";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  monitor: MonitorTile;
  siblings: string[];
  active: boolean;
  onSelect: () => void;
  onArrange: (tile: string, before: string | null) => void;
}

export function MonitorItem({
  machine,
  monitor,
  siblings,
  active,
  onSelect,
  onArrange,
}: Props) {
  const close = useCloseTile();

  return (
    <TileItem
      tile={{ kind: "monitor", machine: machine.id, id: monitor.id }}
      siblings={siblings}
      active={active}
      icon={<PulseIcon />}
      label="Monitor"
      closable
      onSelect={onSelect}
      onArrange={onArrange}
      onClose={() => {
        close.mutate({ machine, tile: monitor.id });
      }}
    />
  );
}
