import type { PullDetail } from "@/generated/protocol";

export function blockers(pull: PullDetail) {
  if (pull.merge === "clean" || pull.merge === "unknown") {
    return [];
  }

  const required = pull.checks.filter((check) => check.required);
  const failed = required.filter((check) => check.state === "failure").length;
  const running = required.filter((check) => check.state === "pending").length;
  const found = [
    pull.merge === "conflicts" && `Resolve the conflicts with ${pull.base}`,
    pull.draft && "Mark it ready for review",
    pull.merge === "behind" && `Update the branch with ${pull.base}`,
    failed > 0 &&
      (failed === 1
        ? "Fix the failing required check"
        : `Fix ${String(failed)} failing required checks`),
    running > 0 &&
      (running === 1
        ? "Wait for a required check to finish"
        : `Wait for ${String(running)} required checks to finish`),
    pull.review === "changes" && "Address the requested changes",
    pull.review === "required" && "Get an approving review",
    pull.merge === "blocked" &&
      pull.threads.length > 0 &&
      (pull.threads.length === 1
        ? "Resolve the open conversation"
        : `Resolve ${String(pull.threads.length)} open conversations`),
  ].filter((reason) => typeof reason === "string");

  if (found.length === 0 && pull.merge === "blocked") {
    return [`Meet the branch rules on ${pull.base}`];
  }

  return found;
}
