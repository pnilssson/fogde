import { StartItem } from "@/features/workspaces/components/start-item";
import { TerminalItem } from "@/features/workspaces/components/terminal-item";
import { useFocused } from "@/features/workspaces/hooks/use-focused";
import { startId, useStart } from "@/features/workspaces/lib/layouts";
import {
  inView,
  type Selection,
  type View,
} from "@/features/workspaces/lib/selection";
import { byAttention } from "@/features/workspaces/lib/status";
import type { Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

export interface RowsProps {
  machine: Machine;
  workspace: Workspace;
  onSelect: (view: View, terminal?: Selection) => void;
  onStart: (view: View, folder: string) => void;
}

interface Props extends RowsProps {
  view: View;
}

export function ViewRows({
  machine,
  workspace,
  view,
  onSelect,
  onStart,
}: Props) {
  const start = useStart(view);
  const focused = useFocused(workspace, view);

  return (
    <>
      {byAttention(
        workspace.terminals.filter((terminal) => inView(view, terminal)),
      ).map((terminal) => (
        <TerminalItem
          key={terminal.id}
          machine={machine}
          terminal={terminal}
          active={focused === terminal.id}
          onSelect={(selection) => {
            onSelect(view, selection);
          }}
        />
      ))}
      {start !== undefined && (
        <StartItem
          active={focused === startId}
          onSelect={() => {
            onStart(view, start);
          }}
        />
      )}
    </>
  );
}
