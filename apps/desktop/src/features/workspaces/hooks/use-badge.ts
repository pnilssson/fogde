import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";

import type { useGroups } from "@/features/workspaces/hooks/use-terminals";
import { needsYou } from "@/features/workspaces/lib/status";
import { frame } from "@/lib/frame";

const current = getCurrentWindow();
const size = 32;

export function useBadge(groups: ReturnType<typeof useGroups>) {
  const count = groups
    .flatMap(({ workspaces }) => workspaces.data ?? [])
    .flatMap(({ terminals }) => terminals)
    .filter(({ status }) => needsYou(status)).length;

  useEffect(() => {
    void show(count).catch(() => undefined);
  }, [count]);
}

async function show(count: number) {
  if (frame.platform !== "windows") {
    await current.setBadgeCount(count > 0 ? count : undefined);
    return;
  }

  await current.setOverlayIcon(count > 0 ? await overlay(count) : undefined);
}

async function overlay(count: number) {
  const canvas = new OffscreenCanvas(size, size);
  const context = canvas.getContext("2d");
  if (!context) {
    throw new Error("Canvas is unavailable");
  }

  const label = count > 9 ? "9+" : String(count);
  const style = getComputedStyle(document.documentElement);

  context.fillStyle = style.getPropertyValue("--destructive").trim();
  context.beginPath();
  context.arc(size / 2, size / 2, size / 2, 0, Math.PI * 2);
  context.fill();

  context.fillStyle = "#ffffff";
  context.font = `600 ${String(label.length > 1 ? 16 : 20)}px sans-serif`;
  context.textAlign = "center";
  context.textBaseline = "middle";
  context.fillText(label, size / 2, size / 2 + 1);

  const blob = await canvas.convertToBlob({ type: "image/png" });

  return new Uint8Array(await blob.arrayBuffer());
}
