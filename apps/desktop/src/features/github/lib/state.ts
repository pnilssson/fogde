import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

import type { Kind } from "@/features/github/api/github";
import type { DiffScope, Repo } from "@/generated/protocol";

export type Tab = "changes" | Kind;

export type Layout = "list" | "folders";

export type DiffStyle = "unified" | "split";

export interface Item {
  kind: Kind;
  repo: Repo;
  number: number;
}

interface State {
  tab: Tab;
  closed: boolean;
  layout: Layout;
  diffStyle: DiffStyle;
  diffScope: DiffScope | undefined;
  item: Partial<Record<string, Item>>;
  drafts: Partial<Record<string, string>>;
  target: string | undefined;
  shown: string | undefined;
}

const github = createStore<State>()(() => ({
  tab: "changes",
  closed: false,
  layout: "list",
  diffStyle: "unified",
  diffScope: undefined,
  item: {},
  drafts: {},
  target: undefined,
  shown: undefined,
}));

export function useGitHubState() {
  const tab = useStore(github, (state) => state.tab);
  const closed = useStore(github, (state) => state.closed);

  return { tab, closed };
}

export function useChosen(machine: string) {
  const pull = useStore(github, (state) => state.item[`${machine}:pull`]);
  const issue = useStore(github, (state) => state.item[`${machine}:issue`]);

  return { pull, issue };
}

export function useDraft(key: string) {
  return useStore(github, ({ drafts }) => drafts[key] ?? "");
}

export function setDraft(key: string, text: string) {
  github.setState(({ drafts }) => ({ drafts: { ...drafts, [key]: text } }));
}

export function setTab(tab: Tab) {
  github.setState({ tab });
}

export function useLayout() {
  return useStore(github, (state) => state.layout);
}

export function setLayout(layout: Layout) {
  github.setState({ layout });
}

export function useDiffStyle() {
  return useStore(github, (state) => state.diffStyle);
}

export function setDiffStyle(diffStyle: DiffStyle) {
  github.setState({ diffStyle });
}

export function useDiffScope(uncommitted: number): DiffScope {
  const chosen = useStore(github, (state) => state.diffScope);

  return uncommitted === 0 ? "branch" : (chosen ?? "uncommitted");
}

export function showScope(diffScope: DiffScope, path?: string) {
  github.setState({ diffScope, target: path });
}

export function toggleClosed() {
  github.setState(({ closed }) => ({ closed: !closed }));
}

export function useTarget() {
  return useStore(github, (state) => state.target);
}

export function arrive() {
  github.setState({ target: undefined });
}

export function useShown() {
  return useStore(github, (state) => state.shown);
}

export function setShown(shown: string | undefined) {
  if (github.getState().shown !== shown) {
    github.setState({ shown });
  }
}

export function select(machine: string, chosen: Item) {
  github.setState(({ item }) => ({
    tab: chosen.kind,
    item: { ...item, [`${machine}:${chosen.kind}`]: chosen },
  }));
}

export function sameRepo(a: Repo, b: Repo) {
  return (
    a.host === b.host &&
    a.owner.toLowerCase() === b.owner.toLowerCase() &&
    a.name.toLowerCase() === b.name.toLowerCase()
  );
}

export function sameItem(a: Item | undefined, b: Item) {
  return (
    a?.kind === b.kind && a.number === b.number && sameRepo(a.repo, b.repo)
  );
}
