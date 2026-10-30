import { sameRepo } from "@/features/github/lib/state";
import type {
  CheckoutStatus,
  Listed,
  Repo,
  Workspace,
} from "@/generated/protocol";

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

export function ago(time: string | number, now = Date.now()) {
  const at = typeof time === "number" ? time : Date.parse(time);
  const minutes = Math.floor((now - at) / 60_000);
  if (minutes < 1) {
    return "just now";
  }
  if (minutes < 60) {
    return `${String(minutes)}m ago`;
  }

  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return `${String(hours)}h ago`;
  }

  const days = Math.floor(hours / 24);
  if (days === 1) {
    return "yesterday";
  }
  if (days < 7) {
    return `${String(days)}d ago`;
  }
  if (days < 30) {
    return `${String(Math.floor(days / 7))}w ago`;
  }

  const months = Math.floor(days / 30);
  if (months < 12) {
    return `${String(months)}mo ago`;
  }

  return `${String(Math.max(1, Math.floor(days / 365)))}y ago`;
}

export function samePath(a: string, b: string) {
  const plain = (path: string) => {
    const slashed = path.replaceAll("\\", "/").replace(/\/+$/, "");

    return /^[a-z]:\//i.test(slashed) ? slashed.toLowerCase() : slashed;
  };

  return plain(a) === plain(b);
}
