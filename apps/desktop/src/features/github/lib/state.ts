import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

import type { Kind } from "@/features/github/api/github";
import type { DiffScope, Repo } from "@/generated/protocol";

export type Scope = "workspace" | "all";

export type Tab = "changes" | Kind;

export type Layout = "list" | "folders";

export type DiffStyle = "unified" | "split";

export interface Item {
  kind: Kind;
  repo: Repo;
  number: number;
}

export type Selected = { kind: "checkout"; path: string } | Item;

interface State {
  scope: Scope;
  tab: Tab;
  closed: boolean;
  layout: Layout;
  diffStyle: DiffStyle;
  diffScope: DiffScope | undefined;
  checkout: Partial<Record<string, string>>;
  item: Partial<Record<string, Item>>;
  drafts: Partial<Record<string, string>>;
  target: string | undefined;
  shown: string | undefined;
}

const github = createStore<State>()(() => ({
  scope: "workspace",
  tab: "changes",
  closed: false,
  layout: "list",
  diffStyle: "unified",
  diffScope: undefined,
  checkout: {},
  item: {},
  drafts: {},
  target: undefined,
  shown: undefined,
}));

export function useGitHubState() {
  const scope = useStore(github, (state) => state.scope);
  const tab = useStore(github, (state) => state.tab);
  const closed = useStore(github, (state) => state.closed);

  return { scope, tab, closed };
}

export function useChosen(machine: string) {
  const checkout = useStore(github, (state) => state.checkout[machine]);
  const pull = useStore(github, (state) => state.item[`${machine}:pull`]);
  const issue = useStore(github, (state) => state.item[`${machine}:issue`]);

  return { checkout, pull, issue };
}

export function useDraft(key: string) {
  return useStore(github, ({ drafts }) => drafts[key] ?? "");
}

export function setDraft(key: string, text: string) {
  github.setState(({ drafts }) => ({ drafts: { ...drafts, [key]: text } }));
}

export function setScope(scope: Scope) {
  github.setState({ scope });
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

export function useDiffOptions() {
  const style = useStore(github, (state) => state.diffStyle);
  const scope = useStore(github, (state) => state.diffScope);

  return { style, scope };
}

export function setDiffStyle(diffStyle: DiffStyle) {
  github.setState({ diffStyle });
}

export function setDiffScope(diffScope: DiffScope) {
  github.setState({ diffScope });
}

export function toggleClosed() {
  github.setState(({ closed }) => ({ closed: !closed }));
}

export function useTarget() {
  return useStore(github, (state) => state.target);
}

export function jump(path: string) {
  github.setState({ target: path });
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

export function select(machine: string, chosen: Selected) {
  if (chosen.kind === "checkout") {
    github.setState(({ checkout }) => ({
      tab: "changes",
      checkout: { ...checkout, [machine]: chosen.path },
      target: undefined,
      shown: undefined,
    }));
    return;
  }

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
