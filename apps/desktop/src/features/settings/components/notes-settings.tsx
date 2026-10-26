import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { open as pick } from "@tauri-apps/plugin-dialog";
import { useState } from "react";

import { FolderDialog } from "@/components/folder-dialog";
import { Button } from "@/components/ui/button";
import { notesFolderQuery } from "@/lib/folders";
import {
  SettingsRow,
  SettingsSection,
} from "@/features/settings/components/settings-section";
import { useMachines } from "@/hooks/use-machines";
import type { Machine } from "@/lib/machines";
import { report } from "@/lib/report";

function failed(error: unknown) {
  report("Could not save the notes folder", error);
}

export function NotesSettings() {
  const { connected } = useMachines();

  return (
    <SettingsSection
      id="notes-heading"
      title="Notes folder"
      hint="Use a folder you already have, like an Obsidian vault."
    >
      {connected.map((machine) => (
        <Row key={machine.id} machine={machine} />
      ))}
    </SettingsSection>
  );
}

function Row({ machine }: { machine: Machine }) {
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
    if (!machine.local) {
      setBrowsing(true);
      return;
    }

    const picked = await pick({ directory: true, defaultPath: folder });
    if (picked) {
      save.mutate(picked);
    }
  };

  return (
    <SettingsRow label={machine.name} detail={folder}>
      <Button
        variant="ghost"
        size="sm"
        aria-label={`Change the notes folder on ${machine.name}`}
        aria-busy={save.isPending}
        disabled={folder === undefined || save.isPending}
        onClick={() => {
          change().catch(failed);
        }}
      >
        {save.isPending ? "Saving…" : "Change"}
      </Button>
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
  );
}
