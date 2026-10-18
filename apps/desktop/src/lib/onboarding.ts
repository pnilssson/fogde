const order = [
  "where",
  "server",
  "github",
  "agents",
  "repos",
  "clone",
] as const;

export function steps(server: boolean) {
  return order.filter((step) => server || step !== "server");
}

export function stepOf(pathname: string) {
  return order.find((step) => pathname === `/onboarding/${step}`);
}

export function stepTransition(from: string | undefined, to: string) {
  const before = from === undefined ? undefined : stepOf(from);
  const after = stepOf(to);
  if (!before || !after || before === after) {
    return false;
  }

  return [order.indexOf(after) > order.indexOf(before) ? "forward" : "back"];
}

export const stepPendingMs = Number.POSITIVE_INFINITY;
