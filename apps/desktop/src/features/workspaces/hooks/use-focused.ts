import { startId, useStart, useTiles } from "@/features/workspaces/lib/layouts";
import {
  inView,
  sameView,
  type View,
} from "@/features/workspaces/lib/selection";
import type { Workspace } from "@/generated/protocol";

export function useFocused(workspace: Workspace, view: View) {
  const here = useTiles((tiles) => sameView(tiles.workspace, view));
  const active = useTiles((tiles) => tiles.active);
  const starting = useTiles((tiles) => tiles.startActive);
  const shared = useStart(view) !== undefined;
  if (!here) {
    return undefined;
  }

  if (starting) {
    return shared ? startId : undefined;
  }

  const listed = workspace.terminals.some(
    (terminal) => terminal.id === active?.id && inView(view, terminal),
  );

  return listed && active?.machine === view.machine ? active.id : undefined;
}
