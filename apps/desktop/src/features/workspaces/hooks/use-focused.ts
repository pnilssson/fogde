import { startId, useStart, useTiles } from "@/features/workspaces/lib/layouts";
import {
  inView,
  sameView,
  type View,
} from "@/features/workspaces/lib/selection";
import { tileId } from "@/features/workspaces/lib/tiles";
import type { Workspace } from "@/generated/protocol";

export function useFocused(workspace: Workspace, view: View) {
  const here = useTiles((tiles) => sameView(tiles.workspace, view));
  const focused = useTiles((tiles) => tiles.focused);
  const shared = useStart(view) !== undefined;
  if (!here) {
    return undefined;
  }

  if (focused === startId) {
    return shared ? startId : undefined;
  }

  return workspace.tiles.find(
    (tile) =>
      inView(view, tile) &&
      tileId({ machine: view.machine, id: tile.id }) === focused,
  )?.id;
}
