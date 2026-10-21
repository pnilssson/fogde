import { useQuery } from "@tanstack/react-query";
import { open as pick } from "@tauri-apps/plugin-dialog";
import { useState } from "react";

import { FolderDialog } from "@/components/folder-dialog";
import { useCreateWorkspace } from "@/features/workspaces/hooks/use-workspaces";
import { reposFolderQuery } from "@/lib/folders";
import type { Machine } from "@/lib/machines";

export function useNewWorkspace(machine: Machine) {
  const [browsing, setBrowsing] = useState(false);
  const create = useCreateWorkspace(machine, () => {
    setBrowsing(false);
  });
  const { data: repos } = useQuery(reposFolderQuery(machine));

  const start = async () => {
    if (!machine.local) {
      setBrowsing(true);
      return;
    }

    const picked = await pick({ directory: true, defaultPath: repos });
    if (picked) {
      create.mutate(picked);
    }
  };

  const dialog = repos !== undefined && (
    <FolderDialog
      machine={machine}
      open={browsing}
      start={repos}
      title="New workspace"
      action="Create workspace"
      busy={create.isPending}
      onOpenChange={setBrowsing}
      onChoose={(path) => {
        create.mutate(path);
      }}
    />
  );

  return { start, dialog };
}
