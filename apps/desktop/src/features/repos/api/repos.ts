import { queryOptions } from "@tanstack/react-query";

import { call } from "@/lib/host";

export const reposFolderQuery = queryOptions({
  queryKey: ["repos.folder"],
  queryFn: () => call("repos.folder"),
});

export function foldersQuery(path: string) {
  return queryOptions({
    queryKey: ["folders.list", path],
    queryFn: () => call("folders.list", path),
  });
}

export function remoteReposQuery(host: string) {
  return queryOptions({
    queryKey: ["repos.list", host],
    queryFn: () => call("repos.list", host),
  });
}
