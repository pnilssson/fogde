import { useNoteTiles, useStart } from "@/features/workspaces/lib/layouts";
import { inView, type View } from "@/features/workspaces/lib/selection";
import type { Workspace } from "@/generated/protocol";

export function useShown(workspace: Workspace, view: View) {
  const start = useStart(view);
  const notes = useNoteTiles(view);

  return (
    workspace.terminals.some((terminal) => inView(view, terminal)) ||
    start !== undefined ||
    notes.length > 0
  );
}
