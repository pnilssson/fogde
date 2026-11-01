import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { ActionButton } from "@/components/action-button";
import { FolderDialog } from "@/components/folder-dialog";
import { notesFolderQuery } from "@/lib/folders";
import {
  SettingsRow,
  SettingsSection,
} from "@/features/settings/components/settings-section";
import type { Machine } from "@/lib/machines";
import { pickFolder } from "@/lib/pick-folder";
import { report } from "@/lib/report";

function failed(error: unknown) {
  report("Could not save the notes folder", error);
}

export function NotesSettings({ machine }: { machine: Machine }) {
  const { data: folder } = useQuery(notesFolderQuery(machine));
  const [browsing, setBrowsing] = useState(false);
  const queries = useQueryClient();
  const save = useMutation({
    mutationFn: (path: string) => machine.client.call("notes.setFolder", path),
    onSuccess: (_, path) => {
      queries.setQueryData(notesFolderQuery(machine).queryKey, path);
      setBrowsing(false);
    },
    onError: failed,
  });

  const change = async () => {
    const picked = await pickFolder(machine, folder, () => {
      setBrowsing(true);
    });
    if (picked) {
      save.mutate(picked);
    }
  };

  return (
    <SettingsSection
      id="notes-heading"
      title="Notes"
      hint="Use a folder you already have, like an Obsidian vault."
    >
      <SettingsRow label="Folder" detail={folder}>
        <ActionButton
          variant="ghost"
          label="Change"
          busy="Saving…"
          pending={save.isPending}
          aria-label="Change the notes folder"
          disabled={folder === undefined}
          onClick={() => {
            change().catch(failed);
          }}
        />
        {folder !== undefined && (
          <FolderDialog
            machine={machine}
            open={browsing}
            start={folder}
            title="Choose a notes folder"
            action="Use this folder"
            busy={save.isPending}
            onOpenChange={setBrowsing}
            onChoose={(path) => {
              save.mutate(path);
            }}
          />
        )}
      </SettingsRow>
    </SettingsSection>
  );
}
