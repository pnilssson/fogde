import type { Status, Terminal } from "@/generated/protocol";

export function needsYou(status: Status) {
  return status === "blocked" || status === "done";
}

export function busy(status: Status) {
  return status === "working" || status === "blocked";
}

export function attention(terminals: Terminal[]): Status {
  const waiting = terminals.map(({ status }) => status).filter(needsYou);

  return waiting.includes("blocked") ? "blocked" : (waiting[0] ?? "idle");
}

export function byAttention(terminals: Terminal[]) {
  return terminals.toSorted(
    (a, b) => Number(needsYou(b.status)) - Number(needsYou(a.status)),
  );
}
