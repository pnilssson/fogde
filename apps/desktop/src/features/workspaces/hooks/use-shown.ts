import { useStart } from "@/features/workspaces/lib/layouts";
import { inView, type View } from "@/features/workspaces/lib/selection";
import type { Workspace } from "@/generated/protocol";

export function useShown(workspace: Workspace, view: View) {
  const start = useStart(view);

  return (
    workspace.tiles.some((tile) => inView(view, tile)) || start !== undefined
  );
}
