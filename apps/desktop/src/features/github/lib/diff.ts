import { type FileDiffMetadata, parsePatchFiles } from "@pierre/diffs";

import type { FileDiff } from "@/generated/protocol";

const generated =
  /(^|\/)(package-lock\.json|npm-shrinkwrap\.json|pnpm-lock\.yaml|yarn\.lock|bun\.lockb?|Cargo\.lock|go\.sum|poetry\.lock|uv\.lock|Gemfile\.lock|composer\.lock|flake\.lock)$|\.min\.(js|css)$|\.map$|(^|\/)(dist|build|vendor|generated)\//;

export function isGenerated(path: string) {
  return generated.test(path);
}

export function parsed(file: FileDiff): FileDiffMetadata | undefined {
  if (!file.patch) {
    return undefined;
  }

  return parsePatchFiles(file.patch, file.path)[0]?.files[0];
}

export function versionOf(file: FileDiff, collapsed: boolean) {
  const patch = file.patch ?? "";
  let hash = 5381;
  for (let index = 0; index < patch.length; index++) {
    hash = Math.imul(hash, 33) ^ patch.charCodeAt(index);
  }

  return hash * 2 + (collapsed ? 1 : 0);
}

export function hiddenReason(file: FileDiff) {
  if (file.binary) {
    return "binary file";
  }
  if (file.large) {
    return "too large to show";
  }
  if (!file.patch) {
    return "over the size limit for one view";
  }

  return null;
}

export interface Viewport {
  top: number;
  height: number;
  remaining: number;
}

const margin = 8;

export function shownFile(
  ids: readonly string[],
  topOf: (id: string) => number | undefined,
  { top, height, remaining }: Viewport,
) {
  const rest = remaining <= 1 ? 0 : remaining;
  const span = Math.min(height, top + rest);
  const progress = span > 0 && rest < span ? 1 - rest / span : 0;
  const line = top + margin + progress * (height - margin);

  let current = ids[0];
  for (const id of ids) {
    const start = topOf(id);
    if (start === undefined || start > line) {
      break;
    }

    current = id;
  }

  return current;
}

export const diffCss =
  "[data-separator]{background-color:var(--diffs-bg-separator)}[data-separator] [data-separator-wrapper],[data-separator] [data-separator-content]{background-color:transparent}";
