import { afterEach, beforeEach, expect, test, vi } from "vitest";

import { NoteSession } from "@/features/notes/lib/session";
import type { Written } from "@/generated/protocol";

class File {
  text: string;
  version = 1;

  constructor(text: string) {
    this.text = text;
  }

  write(text: string, version: string): Written {
    if (version !== String(this.version)) {
      return { kind: "changed" };
    }

    this.text = text;
    this.version++;
    return { kind: "saved", version: String(this.version) };
  }

  agent(text: string) {
    this.text = text;
    this.version++;
  }
}

function open(file: File) {
  let shown = file.text;
  const kept = vi.fn();
  const session: NoteSession = new NoteSession(
    file.text,
    String(file.version),
    {
      text: () => shown,
      replace: (text) => {
        shown = text;
      },
    },
    {
      write: (text, version) => Promise.resolve(file.write(text, version)),
      saved: () => undefined,
      reload: () => {
        session.loaded(file.text, String(file.version));
      },
      kept,
      failed: (error) => {
        throw error;
      },
    },
  );

  return {
    session,
    kept,
    shown: () => shown,
    type: (text: string) => {
      shown = text;
      session.edited();
    },
  };
}

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

test("saves typing after a pause", async () => {
  const file = new File("# Plan\n");
  const editor = open(file);

  editor.type("# Plan\n\n- login\n");
  expect(file.text).toBe("# Plan\n");
  await vi.runAllTimersAsync();

  expect(file.text).toBe("# Plan\n\n- login\n");
});

test("shows an agent's change when nothing is unsaved", () => {
  const file = new File("# Plan\n");
  const editor = open(file);

  file.agent("# Plan\n\n- signup\n");
  editor.session.loaded(file.text, String(file.version));

  expect(editor.shown()).toBe("# Plan\n\n- signup\n");
});

test("merges an agent's change into unsaved typing and saves both", async () => {
  const file = new File("# Plan\n\n- login\n");
  const editor = open(file);

  editor.type("# Plan for login\n\n- login\n");
  file.agent("# Plan\n\n- login\n- signup\n");
  await vi.runAllTimersAsync();

  expect(file.text).toBe("# Plan for login\n\n- login\n- signup\n");
  expect(editor.shown()).toBe(file.text);
  expect(editor.kept).not.toHaveBeenCalled();
});

test("a late reload of its own earlier save leaves newer typing in place", async () => {
  const file = new File("# Plan\n");
  const editor = open(file);

  editor.type("# Plan\n\n- login\n");
  await vi.runAllTimersAsync();
  const earlier = { text: file.text, version: String(file.version) };
  editor.type("# Plan\n\n- login\n- signup\n");
  await vi.runAllTimersAsync();

  editor.session.loaded(earlier.text, earlier.version);

  expect(editor.shown()).toBe("# Plan\n\n- login\n- signup\n");
  expect(file.text).toBe("# Plan\n\n- login\n- signup\n");
});

test("keeps the user's text when an agent changed the same line", async () => {
  const file = new File("# Plan\n\n- login\n");
  const editor = open(file);

  editor.type("# Plan\n\n- login with email\n");
  file.agent("# Plan\n\n- login with SSO\n");
  await vi.runAllTimersAsync();

  expect(file.text).toBe("# Plan\n\n- login with email\n");
  expect(editor.kept).toHaveBeenCalledOnce();
});

test("a failed save waits for the next edit instead of retrying", async () => {
  const failed = vi.fn();
  const write = vi.fn(() => Promise.reject(new Error("gone")));
  let shown = "# Plan\n";
  const session = new NoteSession(
    shown,
    "1",
    {
      text: () => shown,
      replace: (text) => {
        shown = text;
      },
    },
    {
      write,
      saved: () => undefined,
      reload: () => undefined,
      kept: () => undefined,
      failed,
    },
  );

  shown = "# Plan\n\n- login\n";
  session.edited();
  await vi.advanceTimersByTimeAsync(10_000);

  expect(write).toHaveBeenCalledOnce();
  expect(failed).toHaveBeenCalledOnce();

  shown = "# Plan\n\n- login\n- signup\n";
  session.edited();
  await vi.advanceTimersByTimeAsync(10_000);

  expect(write).toHaveBeenCalledTimes(2);
});

test("never writes a note that was only opened", async () => {
  const file = new File("# Plan\r\n");
  const editor = open(file);

  editor.session.flush();
  await vi.runAllTimersAsync();

  expect(file.version).toBe(1);
});
