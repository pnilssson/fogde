import type { Repo } from "@/generated/protocol";

export function repoKey(repo: Repo) {
  return `${repo.host}/${repo.owner}/${repo.name}`;
}
