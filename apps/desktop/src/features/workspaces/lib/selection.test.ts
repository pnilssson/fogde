import { expect, test } from "vitest";

import type { useGroups } from "@/features/workspaces/hooks/use-terminals";
import {
  nextWaiting,
  resolveWorkspace,
} from "@/features/workspaces/lib/selection";
import type { Status } from "@/generated/protocol";

type Groups = ReturnType<typeof useGroups>;

function terminal(id: string, status: Status) {
  return { id, name: id, agent: "claude", status, folder: "", checkout: null };
}

function workspace(id: string, terminals: ReturnType<typeof terminal>[]) {
  return { id, name: id, folder: "", missing: false, checkouts: [], terminals };
}

const groups = [
  {
    machine: { id: "local" },
    workspaces: {
      data: [
        workspace("api", [
          terminal("a", "idle"),
          terminal("b", "done"),
          terminal("c", "blocked"),
        ]),
      ],
    },
  },
  {
    machine: { id: "vps" },
    workspaces: { data: [workspace("web", [terminal("d", "done")])] },
  },
] as unknown as Groups;

function next(active?: string, machine = "local") {
  return nextWaiting(groups, active ? { machine, id: active } : undefined)
    ?.terminal.id;
}

test("goes to each agent that needs you in sidebar order and wraps around", () => {
  expect(next()).toBe("b");
  expect(next("a")).toBe("b");
  expect(next("b")).toBe("c");
  expect(next("c")).toBe("d");
  expect(next("d", "vps")).toBe("b");
});

test("a removed worktree's view falls back to its workspace", () => {
  const stored = { machine: "local", id: "api", checkout: "/worktrees/login" };

  expect(resolveWorkspace(groups, stored)).toEqual({
    machine: "local",
    id: "api",
  });
});

test("finds nothing when no agent needs you", () => {
  const idle = [
    {
      machine: { id: "local" },
      workspaces: { data: [workspace("api", [terminal("a", "working")])] },
    },
  ] as unknown as Groups;

  expect(nextWaiting(idle, undefined)).toBeUndefined();
});
