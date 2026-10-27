import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

import { moved, onRename } from "@/lib/note-renames";

export type Page = "notes" | "monitor";

interface State {
  page: Page | undefined;
  machine: string | undefined;
  notes: Partial<Record<string, string>>;
}

const pages = createStore<State>()(() => ({
  page: undefined,
  machine: undefined,
  notes: {},
}));

onRename(({ machine, from, to }) => {
  pages.setState(({ notes }) => {
    const note = notes[machine];
    const next = note ? moved(note, from, to) : undefined;

    return next ? { notes: { ...notes, [machine]: next } } : {};
  });
});

export function usePage() {
  return useStore(pages, ({ page }) => page);
}

export function usePageMachine() {
  return useStore(pages, ({ machine }) => machine);
}

export function useShownNote(machine: string) {
  return useStore(pages, ({ notes }) => notes[machine]);
}

export function openPage(page: Page, machine: string | undefined) {
  pages.setState({ page, machine });
}

export function openNote(machine: string, note: string) {
  pages.setState(({ notes }) => ({
    page: "notes",
    machine,
    notes: { ...notes, [machine]: note },
  }));
}

export function closePage() {
  pages.setState({ page: undefined });
}

export function leavePage(then: () => void) {
  if (pages.getState().page === undefined) {
    then();
    return;
  }

  closePage();
  requestAnimationFrame(then);
}

export function pageOpen() {
  return pages.getState().page !== undefined;
}
