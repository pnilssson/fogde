import { queryOptions } from "@tanstack/react-query";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow, UserAttentionType } from "@tauri-apps/api/window";
import {
  isPermissionGranted,
  onAction,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

import { callOwn } from "@/lib/host";

export const notificationsQuery = queryOptions({
  queryKey: ["notifications.get"],
  queryFn: () => callOwn("notifications.get"),
});

const kept = 50;
const clicks = new Map<number, () => void>();
let next = 0;
let listening: Promise<unknown> | undefined;

function listen() {
  listening ??= onAction(({ actionId, notification }) => {
    if (actionId !== "tap" || !notification) {
      return;
    }

    const click = clicks.get(notification.id);
    clicks.delete(notification.id);
    click?.();
  });

  return listening;
}

async function allowed() {
  return (
    (await isPermissionGranted()) || (await requestPermission()) === "granted"
  );
}

interface Alert {
  title: string;
  body: string;
  onClick: () => void;
}

export async function notify({ title, body, onClick }: Alert) {
  if (!isTauri() || !(await allowed())) {
    return;
  }

  await listen();
  next += 1;
  clicks.set(next, onClick);
  for (const id of [...clicks.keys()].slice(0, -kept)) {
    clicks.delete(id);
  }

  sendNotification({
    id: next,
    title,
    body,
  });
  await getCurrentWindow().requestUserAttention(
    UserAttentionType.Informational,
  );
}

export async function bringForward() {
  const window = getCurrentWindow();
  await window.unminimize();
  await window.setFocus();
}
