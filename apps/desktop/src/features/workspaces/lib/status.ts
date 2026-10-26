import type { Status, Terminal } from "@/generated/protocol";

interface Traits {
  needsYou: boolean;
  busy: boolean;
  rank: number;
  activity: string;
}

const traits: Record<Status, Traits> = {
  idle: { needsYou: false, busy: false, rank: 0, activity: "is open" },
  working: {
    needsYou: false,
    busy: true,
    rank: 1,
    activity: "is still working",
  },
  done: { needsYou: true, busy: false, rank: 2, activity: "has finished" },
  blocked: {
    needsYou: true,
    busy: true,
    rank: 3,
    activity: "is waiting for your input",
  },
};

export function needsYou(status: Status) {
  return traits[status].needsYou;
}

export function busy(status: Status) {
  return traits[status].busy;
}

export function activity(status: Status) {
  return traits[status].activity;
}

export function attention(terminals: Terminal[]): Status {
  return terminals
    .map(({ status }) => status)
    .filter(needsYou)
    .reduce<Status>(
      (top, status) => (traits[status].rank > traits[top].rank ? status : top),
      "idle",
    );
}

export function byAttention(terminals: Terminal[]) {
  return terminals.toSorted(
    (a, b) => Number(needsYou(b.status)) - Number(needsYou(a.status)),
  );
}
