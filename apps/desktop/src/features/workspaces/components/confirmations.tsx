import { useContext } from "react";

import { ConfirmDialog } from "@/components/confirm-dialog";
import { useCloseTerminal } from "@/features/workspaces/hooks/use-terminals";
import { useRemoveWorktree } from "@/features/workspaces/hooks/use-worktrees";
import { checkoutName, working } from "@/features/workspaces/lib/checkouts";
import { setAsking, useAsked } from "@/features/workspaces/lib/confirm";
import {
  findTerminal,
  findWorkspace,
  WorkspaceContext,
} from "@/features/workspaces/lib/selection";
import { activity } from "@/features/workspaces/lib/status";
import type { Checkout, Terminal, Workspace } from "@/generated/protocol";
import { agents } from "@/lib/agents";
import type { Machine } from "@/lib/machines";

function closing({ agent, status }: Terminal) {
  const who = agent ? agents[agent].name : "The terminal";

  return `${who} ${activity(status)}. Closing stops it.`;
}

function removing(terminals: Terminal[]) {
  const outcome = "Removing it closes its terminals.";
  if (terminals.length > 1) {
    return `${String(terminals.length)} terminals are still busy. ${outcome}`;
  }

  const [terminal] = terminals;
  if (!terminal) {
    return outcome;
  }

  const who = terminal.agent ? agents[terminal.agent].name : "Its terminal";

  return `${who} ${activity(terminal.status)}. ${outcome}`;
}

interface CloseProps {
  machine: Machine;
  terminal: Terminal;
  open: boolean;
}

function CloseTerminal({ machine, terminal: asked, open }: CloseProps) {
  const close = useCloseTerminal(machine);
  const { groups } = useContext(WorkspaceContext);
  const terminal =
    findTerminal(groups, { machine: machine.id, id: asked.id })?.terminal ??
    asked;

  return (
    <ConfirmDialog
      open={open}
      onOpenChange={setAsking}
      title={`Close ${terminal.name}?`}
      description={closing(terminal)}
      action="Close"
      onConfirm={() => {
        close.mutate(terminal);
        setAsking(false);
      }}
    />
  );
}

interface RemoveProps {
  machine: Machine;
  workspace: Workspace;
  checkout: Checkout;
  open: boolean;
}

function RemoveWorktree({
  machine,
  workspace: asked,
  checkout,
  open,
}: RemoveProps) {
  const { groups } = useContext(WorkspaceContext);
  const workspace =
    findWorkspace(groups, { machine: machine.id, id: asked.id })?.workspace ??
    asked;
  const remove = useRemoveWorktree(machine, workspace);

  return (
    <ConfirmDialog
      open={open}
      onOpenChange={setAsking}
      title={`Remove ${checkoutName(checkout)}?`}
      description={removing(working(workspace, checkout.path))}
      action="Remove"
      onConfirm={() => {
        remove.mutate(checkout.path);
        setAsking(false);
      }}
    />
  );
}

export function Confirmations() {
  const { asked, open } = useAsked();
  if (!asked) {
    return null;
  }

  if (asked.kind === "close") {
    return (
      <CloseTerminal
        machine={asked.machine}
        terminal={asked.terminal}
        open={open}
      />
    );
  }

  return (
    <RemoveWorktree
      machine={asked.machine}
      workspace={asked.workspace}
      checkout={asked.checkout}
      open={open}
    />
  );
}
