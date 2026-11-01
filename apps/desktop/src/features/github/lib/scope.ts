import { sameRepo } from "@/features/github/lib/state";
import type {
  Account,
  CheckoutStatus,
  DiffScope,
  Listed,
  Repo,
  Workspace,
} from "@/generated/protocol";
import { samePath } from "@/lib/folders";

export const nowhere: Repo = { host: "", owner: "", name: "" };

export interface Entry {
  repo: Repo;
  item: Listed;
}

export function pathsOf(workspaces: Workspace[]) {
  const paths = workspaces.flatMap(({ checkouts }) =>
    checkouts.map(({ path }) => path),
  );

  return [...new Set(paths)].sort();
}

export function grouped(workspaces: Workspace[], statuses: CheckoutStatus[]) {
  return workspaces
    .map((workspace) => ({
      workspace,
      checkouts: statuses.filter((status) =>
        workspace.checkouts.some(({ path }) => samePath(path, status.path)),
      ),
    }))
    .filter(({ checkouts }) => checkouts.length > 0);
}

export function entriesOf(repo: Repo | undefined, items: Listed[] | undefined) {
  return repo ? (items ?? []).map((item) => ({ repo, item })) : [];
}

interface Head {
  head: string;
  owner: string | null;
  cross: boolean;
}

export function holds(checkout: CheckoutStatus, number: number, pull: Head) {
  if (checkout.pull === number) {
    return true;
  }
  if (!checkout.owner || !pull.owner) {
    return !pull.cross && checkout.branch === pull.head;
  }

  return (
    checkout.owner.toLowerCase() === pull.owner.toLowerCase() &&
    (checkout.branch === pull.head ||
      (pull.cross && checkout.branch === `${pull.owner}/${pull.head}`))
  );
}

export function pullFor(entries: Entry[], checkout: CheckoutStatus) {
  return entries.find(
    (entry) => entry.item.state === "open" && belongs(entry, checkout),
  );
}

export function landed(entries: Entry[], checkout: CheckoutStatus) {
  const head = checkout.last?.sha;
  if (!head || checkout.files.length > 0) {
    return undefined;
  }

  return entries.find(
    (entry) =>
      entry.item.state === "merged" &&
      entry.item.pull?.sha === head &&
      entry.item.pull.base !== checkout.branch &&
      belongs(entry, checkout),
  );
}

function belongs({ repo: base, item }: Entry, checkout: CheckoutStatus) {
  const { pull } = item;
  if (!pull || !checkout.repo || !sameRepo(base, checkout.repo)) {
    return false;
  }

  const cross = pull.owner?.toLowerCase() !== base.owner.toLowerCase();

  return holds(checkout, item.number, { ...pull, cross });
}

export function authored(
  accounts: Account[] | undefined,
  host: string,
  author: string | null,
) {
  const login = author?.toLowerCase();

  return (
    accounts?.some(
      (account) =>
        account.host === host && account.login.toLowerCase() === login,
    ) === true
  );
}

export function noChanges(scope: DiffScope, base: string | null) {
  if (scope === "turn") {
    return "No changes in the last turn";
  }
  if (scope === "uncommitted") {
    return "No uncommitted changes";
  }

  return `No changes${base ? ` since ${base}` : ""}`;
}
