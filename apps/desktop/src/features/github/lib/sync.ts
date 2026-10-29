import type { CheckoutStatus } from "@/generated/protocol";

export type Sync =
  | { kind: "detached" }
  | { kind: "publish" }
  | { kind: "diverged"; ahead: number; behind: number }
  | { kind: "pull"; behind: number }
  | { kind: "push"; ahead: number }
  | { kind: "fetch" };

export function syncOf({
  branch,
  upstream,
  ahead,
  behind,
}: CheckoutStatus): Sync {
  if (!branch) {
    return { kind: "detached" };
  }
  if (!upstream) {
    return { kind: "publish" };
  }
  if (ahead > 0 && behind > 0) {
    return { kind: "diverged", ahead, behind };
  }
  if (behind > 0) {
    return { kind: "pull", behind };
  }
  if (ahead > 0) {
    return { kind: "push", ahead };
  }

  return { kind: "fetch" };
}

export type Next = "commit" | "sync" | "pull-request" | null;

export function nextStep(
  checkout: CheckoutStatus,
  hasPull: boolean,
  canOpenPull: boolean,
): Next {
  const sync = syncOf(checkout);
  if (checkout.files.length > 0) {
    return "commit";
  }
  if (sync.kind === "publish" || sync.kind === "push" || sync.kind === "pull") {
    return "sync";
  }
  if (!hasPull && canOpenPull) {
    return "pull-request";
  }

  return null;
}
