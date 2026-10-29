import type { ChecksSummary, ReviewState } from "@/generated/protocol";

export function checksLabel({ state, passed, failed, total }: ChecksSummary) {
  if (total === 0) {
    return state === "success"
      ? "Checks passed"
      : state === "failure"
        ? "Checks failed"
        : "Checks running";
  }
  if (failed > 0) {
    return failed === 1
      ? `1 of ${String(total)} checks failed`
      : `${String(failed)} of ${String(total)} checks failed`;
  }
  if (passed < total) {
    return `${String(passed)} of ${String(total)} checks passed, the rest running`;
  }

  return total === 1 ? "1 check passed" : `All ${String(total)} checks passed`;
}

export function reviewLabel(review: ReviewState | null) {
  switch (review) {
    case "approved":
      return "Approved";
    case "changes":
      return "Changes requested";
    case "required":
      return "Review required";
    case null:
      return null;
  }
}
