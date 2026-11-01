import { open as pick } from "@tauri-apps/plugin-dialog";

import type { Machine } from "@/lib/machines";

export async function pickFolder(
  machine: Machine,
  start: string | undefined,
  browse: () => void,
) {
  if (!machine.local) {
    browse();
    return undefined;
  }

  const picked = await pick({
    directory: true,
    defaultPath: start === "" ? undefined : start,
  });

  return picked ?? undefined;
}
