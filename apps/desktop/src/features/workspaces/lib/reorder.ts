import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

import type { Edge } from "@/features/workspaces/lib/drag";
import type { Selection } from "@/features/workspaces/lib/selection";

const reorderType = "application/x-fogde-workspace";

interface Over {
  id: string;
  edge: Edge;
}

const reordering = createStore<{
  moving: Selection | undefined;
  over: Over | undefined;
}>()(() => ({ moving: undefined, over: undefined }));

export function useMoving() {
  return useStore(reordering, ({ moving }) => moving);
}

export function useOverWorkspace(id: string) {
  return useStore(reordering, ({ over }) =>
    over?.id === id ? over.edge : undefined,
  );
}

export function hoverWorkspace(over: Over | undefined) {
  const current = reordering.getState().over;
  if (current?.id === over?.id && current?.edge === over?.edge) {
    return;
  }

  reordering.setState({ over });
}

export function reorderProps(workspace: Selection) {
  return {
    draggable: true,
    onDragStart: (event: { dataTransfer: DataTransfer }) => {
      event.dataTransfer.effectAllowed = "move";
      event.dataTransfer.setData(reorderType, workspace.id);
      reordering.setState({ moving: workspace });
    },
    onDragEnd: () => {
      reordering.setState({ moving: undefined, over: undefined });
    },
  };
}
