import type { Tile, Workspace } from "@/generated/protocol";
import { folderName } from "@/lib/folders";

export type TerminalTile = Extract<Tile, { kind: "terminal" }>;
export type NoteTile = Extract<Tile, { kind: "note" }>;
export type MonitorTile = Extract<Tile, { kind: "monitor" }>;

export function terminalsOf(workspace: Workspace) {
  return workspace.tiles.filter(
    (tile): tile is TerminalTile => tile.kind === "terminal",
  );
}

export function noteTitle(note: string | null | undefined) {
  return note ? folderName(note).replace(/\.md$/i, "") : "Notes";
}
