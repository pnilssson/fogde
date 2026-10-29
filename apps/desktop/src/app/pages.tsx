import { useQuery } from "@tanstack/react-query";
import { useEffect, useRef } from "react";

import { CopiedNotice } from "@/components/copied-notice";
import { MachinePicker } from "@/components/machine-picker";
import { FullPage, PageFrame } from "@/components/page-frame";
import { GitHubView } from "@/features/github/components/github-view";
import { MonitorView } from "@/features/monitor/components/monitor-view";
import { notesQuery } from "@/features/notes/api/notes";
import { NotesView } from "@/features/notes/components/notes-view";
import { SettingsView } from "@/features/settings/components/settings-view";
import { copyPath, usePathCopied } from "@/features/notes/lib/copy";
import { inDialog } from "@/features/workspaces/hooks/use-shortcuts";
import type { useGroups } from "@/features/workspaces/hooks/use-terminals";
import type { View } from "@/features/workspaces/lib/selection";
import { showTiles } from "@/features/workspaces/lib/tiles";
import type { Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";
import {
  openNote,
  openPage,
  usePage,
  usePageMachine,
  useShownNote,
} from "@/lib/page";
import { pressed, shortcuts } from "@/lib/shortcuts";

type Groups = ReturnType<typeof useGroups>;

interface Props {
  groups: Groups;
  workspace: View | undefined;
}

export function Pages({ groups, workspace }: Props) {
  const page = usePage();
  const wanted = usePageMachine();
  const machines = groups.map(({ machine }) => machine);
  if (!page) {
    return null;
  }

  if (page === "settings") {
    return (
      <FullPage title="Settings">
        <SettingsView />
      </FullPage>
    );
  }

  const machine =
    machines.find(({ id }) => id === wanted) ??
    machines.find(({ id }) => id === workspace?.machine) ??
    machines[0];
  if (!machine) {
    return null;
  }

  const frame = {
    machine,
    machines,
    onMachine: (next: string) => {
      openPage(page, next);
    },
  };

  const workspaces =
    groups.find((group) => group.machine.id === machine.id)?.workspaces.data ??
    [];

  if (page === "github") {
    return (
      <GitHubPage
        key={machine.id}
        {...frame}
        workspaces={workspaces}
        current={
          workspace?.machine === machine.id
            ? workspaces.find(({ id }) => id === workspace.id)
            : undefined
        }
      />
    );
  }

  return page === "notes" ? (
    <NotesPage key={machine.id} {...frame} />
  ) : (
    <MonitorPage {...frame} workspaces={workspaces} />
  );
}

interface FrameProps {
  machine: Machine;
  machines: Machine[];
  onMachine: (machine: string) => void;
}

function NotesPage(picker: FrameProps) {
  const { machine } = picker;
  const note = useShownNote(machine.id);
  const list = useQuery(notesQuery(machine));
  const shown = list.data?.notes.some(({ path }) => path === note)
    ? note
    : undefined;
  const copied = usePathCopied();

  useCopyShortcut(
    shown
      ? () => {
          copyPath(shown);
        }
      : undefined,
  );

  return (
    <FullPage title="Notes">
      <NotesView
        machine={machine}
        note={note}
        picker={<MachinePicker {...picker} />}
        onOpen={(next) => {
          openNote(machine.id, next);
        }}
      />
      <CopiedNotice copied={copied} label="Path copied" />
    </FullPage>
  );
}

function GitHubPage({
  workspaces,
  current,
  ...picker
}: FrameProps & { workspaces: Workspace[]; current: Workspace | undefined }) {
  return (
    <FullPage title="GitHub">
      <GitHubView
        machine={picker.machine}
        workspaces={workspaces}
        current={current}
        picker={<MachinePicker {...picker} />}
      />
    </FullPage>
  );
}

function MonitorPage({
  workspaces,
  ...picker
}: FrameProps & { workspaces: Workspace[] }) {
  return (
    <PageFrame title="Monitor" onClose={showTiles}>
      <MonitorView
        machine={picker.machine}
        workspaces={workspaces}
        picker={<MachinePicker {...picker} named />}
      />
    </PageFrame>
  );
}

function useCopyShortcut(action: (() => void) | undefined) {
  const current = useRef(action);

  useEffect(() => {
    current.current = action;
  }, [action]);

  useEffect(() => {
    const down = (event: KeyboardEvent) => {
      if (
        !pressed(event, shortcuts.path) ||
        inDialog(event.target) ||
        (event.target instanceof Element &&
          event.target.closest('[role="menu"]')) ||
        !current.current
      ) {
        return;
      }

      event.preventDefault();
      event.stopPropagation();
      current.current();
    };

    window.addEventListener("keydown", down, { capture: true });

    return () => {
      window.removeEventListener("keydown", down, { capture: true });
    };
  }, []);
}
