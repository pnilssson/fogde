import { useQuery } from "@tanstack/react-query";
import { useEffect, useRef } from "react";

import { Keys } from "@/components/keys";
import { PageFrame } from "@/components/page-frame";
import { Button } from "@/components/ui/button";
import { toast } from "@/components/ui/toast";
import { MonitorView } from "@/features/monitor/components/monitor-view";
import { notesQuery } from "@/features/notes/api/notes";
import { ListToggle } from "@/features/notes/components/list-toggle";
import { NotesView } from "@/features/notes/components/notes-view";
import { inDialog } from "@/features/workspaces/hooks/use-shortcuts";
import type { useGroups } from "@/features/workspaces/hooks/use-terminals";
import { store, useTiles } from "@/features/workspaces/lib/layouts";
import type { View } from "@/features/workspaces/lib/selection";
import { show } from "@/features/workspaces/lib/tiles";
import type { Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";
import {
  leavePage,
  openNote,
  openPage,
  usePage,
  usePageMachine,
  useShownNote,
} from "@/lib/page";
import { pastePath } from "@/lib/paste";
import { report } from "@/lib/report";
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
    onClose: back,
  };

  return page === "notes" ? (
    <NotesPage key={machine.id} {...frame} />
  ) : (
    <MonitorPage
      {...frame}
      workspaces={
        groups.find((group) => group.machine.id === machine.id)?.workspaces
          .data ?? []
      }
    />
  );
}

interface FrameProps {
  machine: Machine;
  machines: Machine[];
  onMachine: (machine: string) => void;
  onClose: () => void;
}

function NotesPage(frame: FrameProps) {
  const { machine } = frame;
  const note = useShownNote(machine.id);
  const list = useQuery(notesQuery(machine));
  const shown = list.data?.notes.some(({ path }) => path === note)
    ? note
    : undefined;
  const target = useTiles(({ active }) => active);
  const insert = (path: string) => {
    if (target?.machine !== machine.id) {
      toast.add({
        type: "error",
        title: "No terminal to insert into",
        description: `Click a terminal on ${machine.name} first.`,
      });
      return;
    }

    machine.client
      .call("terminals.input", { id: target.id, data: pastePath(path) })
      .then(() => {
        leavePage(() => {
          show(target);
        });
      })
      .catch((error: unknown) => {
        report("Could not insert the note path", error);
      });
  };

  useInsertShortcut(
    shown
      ? () => {
          insert(shown);
        }
      : undefined,
  );

  return (
    <PageFrame
      title="Notes"
      {...frame}
      leading={<ListToggle />}
      actions={
        shown && (
          <Button
            variant="secondary"
            size="xs"
            onClick={() => {
              insert(shown);
            }}
          >
            Insert path
            <Keys
              letter={shortcuts.insert}
              className="ml-1 bg-transparent px-0"
            />
          </Button>
        )
      }
    >
      <NotesView
        machine={machine}
        note={note}
        onOpen={(next) => {
          openNote(machine.id, next);
        }}
        onInsert={insert}
      />
    </PageFrame>
  );
}

function MonitorPage({
  workspaces,
  ...frame
}: FrameProps & { workspaces: Workspace[] }) {
  return (
    <PageFrame title="Monitor" {...frame}>
      <MonitorView machine={frame.machine} workspaces={workspaces} />
    </PageFrame>
  );
}

function back() {
  leavePage(() => {
    const { active } = store.getState();
    if (active) {
      show(active);
    }
  });
}

function useInsertShortcut(action: (() => void) | undefined) {
  const current = useRef(action);

  useEffect(() => {
    current.current = action;
  }, [action]);

  useEffect(() => {
    const down = (event: KeyboardEvent) => {
      if (
        !pressed(event, shortcuts.insert) ||
        inDialog(event.target) ||
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
