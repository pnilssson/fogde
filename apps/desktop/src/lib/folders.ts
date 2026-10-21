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

function parts(path: string) {
  const windows = /^[a-z]:/i.test(path);

  return path
    .split(/[\\/]/)
    .filter((part) => part.length > 0)
    .map((part) => (windows ? part.toLowerCase() : part));
}

export function inside(path: string, root: string) {
  const outer = parts(root);
  const inner = parts(path);

  return (
    outer.length > 0 && outer.every((part, index) => inner[index] === part)
  );
}

export function folderName(path: string) {
  return path.split(/[\\/]/).findLast((part) => part.length > 0) ?? path;
}

export function relative(path: string, root: string) {
  if (!inside(path, root)) {
    return undefined;
  }

  const separator = path.includes("\\") ? "\\" : "/";

  return path
    .split(/[\\/]/)
    .filter((part) => part.length > 0)
    .slice(parts(root).length)
    .join(separator);
}
