import { queryOptions } from "@tanstack/react-query";

import type { Machine } from "@/lib/machines";

export const reposFolderQuery = (machine: Machine) =>
  queryOptions({
    queryKey: [machine.id, "repos.folder"],
    queryFn: () => machine.client.call("repos.folder"),
  });

export const foldersQuery = (machine: Machine, path: string) =>
  queryOptions({
    queryKey: [machine.id, "folders.list", path],
    queryFn: () => machine.client.call("folders.list", path),
  });

export function folderName(path: string) {
  return path.split(/[\\/]/).findLast((part) => part.length > 0) ?? path;
}
