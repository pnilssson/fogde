import { queryOptions } from "@tanstack/react-query";
import { isTauri } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";

const hour = 60 * 60 * 1000;

let ready: Update | null = null;

export const updateQuery = queryOptions({
  queryKey: ["update"],
  queryFn: async () => {
    const update = await check();
    if (!update) {
      return ready;
    }

    if (update.version === ready?.version) {
      await update.close();
      return ready;
    }

    await update.download();
    await ready?.close();
    ready = update;

    return update;
  },
  enabled: import.meta.env.PROD && isTauri(),
  retry: false,
  staleTime: Infinity,
  refetchInterval: hour,
});

export async function restart(update: Update) {
  await update.install();
  await relaunch();
}
