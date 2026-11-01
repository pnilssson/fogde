import { useQuery } from "@tanstack/react-query";
import { lazy, Suspense } from "react";

import { CopiedNotice } from "@/components/copied-notice";
import { Loading } from "@/components/loading";
import { MachinePicker } from "@/components/machine-picker";
import { FullPage, PageFrame } from "@/components/page-frame";
import { OpenCheckout } from "@/features/github/lib/open";
import { setTab } from "@/features/github/lib/state";
import { notesQuery } from "@/features/notes/api/notes";
import { copiedPath, copyPath } from "@/features/notes/lib/copy";
import { inDialog, inMenu } from "@/lib/dialogs";
import { useWindowKey } from "@/hooks/use-window-key";
import type { useGroups } from "@/features/workspaces/hooks/use-terminals";
import { mainCheckout } from "@/features/workspaces/lib/checkouts";
import type { View } from "@/features/workspaces/lib/selection";
import { selectWorkspace, showTiles } from "@/features/workspaces/lib/tiles";
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
import { useCopied } from "@/lib/copied";
import { samePath } from "@/lib/folders";

const GitHubView = lazy(() =>
  import("@/features/github/components/github-view").then((module) => ({
    default: module.GitHubView,
  })),
);
const MonitorView = lazy(() =>
  import("@/features/monitor/components/monitor-view").then((module) => ({
    default: module.MonitorView,
  })),
);
const NotesView = lazy(() =>
  import("@/features/notes/components/notes-view").then((module) => ({
    default: module.NotesView,
  })),
);
const SettingsView = lazy(() =>
  import("@/features/settings/components/settings-view").then((module) => ({
    default: module.SettingsView,
  })),
);

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
      <FullPage title="Settings" onClose={showTiles}>
        <Suspense fallback={<Loading />}>
          <SettingsView />
        </Suspense>
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
        view={workspace?.machine === machine.id ? workspace : undefined}
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
  const copied = useCopied(copiedPath);

  useWindowKey((event) =>
    shown &&
    pressed(event, shortcuts.path) &&
    !inDialog(event.target) &&
    !inMenu(event.target)
      ? () => {
          copyPath(shown);
        }
      : undefined,
  );

  return (
    <FullPage title="Notes" onClose={showTiles}>
      <Suspense fallback={<Loading />}>
        <NotesView
          machine={machine}
          note={note}
          picker={<MachinePicker {...picker} />}
          onOpen={(next) => {
            openNote(machine.id, next);
          }}
        />
      </Suspense>
      <CopiedNotice copied={copied} label="Path copied" />
    </FullPage>
  );
}

function GitHubPage({
  workspaces,
  view,
  ...picker
}: FrameProps & { workspaces: Workspace[]; view: View | undefined }) {
  const machine = picker.machine.id;
  const current = workspaces.find(({ id }) => id === view?.id);
  const viewed = current?.checkouts.find(
    ({ path }) => view?.checkout !== undefined && samePath(path, view.checkout),
  );
  const wanted = (viewed ?? (current && mainCheckout(current)))?.path;
  const open = (path: string, near = path) => {
    const owners = workspaces.filter(({ checkouts }) =>
      checkouts.some((checkout) => samePath(checkout.path, near)),
    );
    const owner = owners.find(({ id }) => id === current?.id) ?? owners[0];
    if (!owner) {
      return;
    }

    const listed = owner.checkouts.find((checkout) =>
      samePath(checkout.path, path),
    );
    setTab("changes");
    selectWorkspace(
      listed?.main
        ? { machine, id: owner.id }
        : { machine, id: owner.id, checkout: listed?.path ?? path },
    );
  };

  return (
    <FullPage title="GitHub">
      <OpenCheckout value={open}>
        <Suspense fallback={<Loading />}>
          <GitHubView
            machine={picker.machine}
            workspaces={workspaces}
            wanted={wanted}
            picker={<MachinePicker {...picker} />}
          />
        </Suspense>
      </OpenCheckout>
    </FullPage>
  );
}

function MonitorPage({
  workspaces,
  ...picker
}: FrameProps & { workspaces: Workspace[] }) {
  return (
    <PageFrame title="Monitor" onClose={showTiles}>
      <Suspense fallback={<Loading />}>
        <MonitorView
          machine={picker.machine}
          workspaces={workspaces}
          picker={<MachinePicker {...picker} named />}
        />
      </Suspense>
    </PageFrame>
  );
}
