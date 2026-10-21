import { GitBranchIcon } from "@phosphor-icons/react";

import {
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar";
import { StartItem } from "@/features/workspaces/components/start-item";
import { TerminalItem } from "@/features/workspaces/components/terminal-item";
import { checkoutName } from "@/features/workspaces/lib/checkouts";
import { useStart } from "@/features/workspaces/lib/layouts";
import { byAttention } from "@/features/workspaces/lib/status";
import type { Selection } from "@/features/workspaces/lib/selection";
import type { Terminal, Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  workspace: Workspace;
  onSelect: (terminal: Selection) => void;
  onStart: (folder: string) => void;
}

export function WorkspaceTree({
  machine,
  workspace,
  onSelect,
  onStart,
}: Props) {
  const start = useStart({ machine: machine.id, id: workspace.id });

  const terminals = (list: Terminal[]) =>
    byAttention(list).map((terminal) => (
      <TerminalItem
        key={terminal.id}
        machine={machine}
        terminal={terminal}
        onSelect={onSelect}
      />
    ));

  const starting = (shown: boolean) =>
    start !== undefined &&
    shown && (
      <StartItem
        onSelect={() => {
          onStart(start);
        }}
      />
    );

  if (workspace.checkouts.length < 2) {
    return (
      (workspace.terminals.length > 0 || start !== undefined) && (
        <SidebarMenuSub>
          {terminals(workspace.terminals)}
          {starting(true)}
        </SidebarMenuSub>
      )
    );
  }

  const paths = new Set(workspace.checkouts.map(({ path }) => path));
  const loose = workspace.terminals.filter(
    ({ checkout }) => checkout === null || !paths.has(checkout),
  );

  return (
    <SidebarMenuSub>
      {workspace.checkouts.map((checkout) => {
        const inside = workspace.terminals.filter(
          (terminal) => terminal.checkout === checkout.path,
        );
        const here = start === checkout.path;

        return (
          <SidebarMenuSubItem key={checkout.path}>
            <SidebarMenuSubButton
              render={<button type="button" />}
              onClick={() => {
                onStart(checkout.path);
              }}
            >
              <GitBranchIcon />
              <span>{checkoutName(checkout)}</span>
            </SidebarMenuSubButton>
            {(inside.length > 0 || here) && (
              <SidebarMenuSub>
                {terminals(inside)}
                {starting(here)}
              </SidebarMenuSub>
            )}
          </SidebarMenuSubItem>
        );
      })}
      {terminals(loose)}
      {starting(start !== undefined && !paths.has(start))}
    </SidebarMenuSub>
  );
}
