import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { FolderDialog } from "@/components/folder-dialog";
import { useCreateWorkspace } from "@/features/workspaces/hooks/use-workspaces";
import { reposFolderQuery } from "@/lib/folders";
import type { Machine } from "@/lib/machines";
import { pickFolder } from "@/lib/pick-folder";
import { report } from "@/lib/report";

export function useNewWorkspace(machine: Machine) {
  const queries = useQueryClient();
  const [browsing, setBrowsing] = useState(false);
  const create = useCreateWorkspace(machine, () => {
    setBrowsing(false);
  });
  const { data: repos } = useQuery(reposFolderQuery(machine));

  const start = async () => {
    try {
      const folder = machine.local
        ? (repos ?? (await queries.query(reposFolderQuery(machine))))
        : undefined;
      const picked = await pickFolder(machine, folder, () => {
        setBrowsing(true);
      });
      if (picked) {
        create.mutate(picked);
      }
    } catch (error) {
      report("Could not browse the folders", error);
    }
  };

  const dialog = (
    <FolderDialog
      machine={machine}
      open={browsing}
      start={repos ?? "/"}
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
