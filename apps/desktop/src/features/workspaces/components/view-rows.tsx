import { StartItem } from "@/features/workspaces/components/start-item";
import { TerminalItem } from "@/features/workspaces/components/terminal-item";
import { useStart } from "@/features/workspaces/lib/layouts";
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

  return (
    <>
      {byAttention(
        workspace.terminals.filter((terminal) => inView(view, terminal)),
      ).map((terminal) => (
        <TerminalItem
          key={terminal.id}
          machine={machine}
          terminal={terminal}
          onSelect={(selection) => {
            onSelect(view, selection);
          }}
        />
      ))}
      {start !== undefined && (
        <StartItem
          onSelect={() => {
            onStart(view, start);
          }}
        />
      )}
    </>
  );
}
