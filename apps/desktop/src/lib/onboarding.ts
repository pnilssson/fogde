export const steps = ["github", "agents", "repos", "clone"] as const;

export function stepIndex(pathname: string) {
  return steps.findIndex((step) => pathname === `/onboarding/${step}`);
}

export function stepTransition(from: string | undefined, to: string) {
  const before = from === undefined ? -1 : stepIndex(from);
  const after = stepIndex(to);
  if (before < 0 || after < 0 || before === after) {
    return false;
  }

  return [after > before ? "forward" : "back"];
}
