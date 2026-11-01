import { merge } from "@/features/notes/lib/merge";
import type { NoteText, Written } from "@/generated/protocol";

const pause = 600;
const rounds = 5;
const open = new Set<NoteSession>();

export async function saveAll() {
  const saved = await Promise.all([...open].map((session) => session.finish()));

  return saved.every(Boolean);
}

export interface Resumed {
  base: string;
  version: string;
  text: string;
}

export function resume(key: string): Resumed | undefined {
  for (const session of open) {
    const left = session.leftover(key);
    if (left) {
      open.delete(session);
      return left;
    }
  }

  return undefined;
}

export interface Disk {
  write: (text: string, version: string) => Promise<Written>;
  read: () => Promise<NoteText>;
  saved: (text: string, version: string) => void;
  kept: () => void;
  failed: (error: unknown) => void;
}

export interface Document {
  text: () => string;
  replace: (text: string) => void;
}

export class NoteSession {
  private base: string;
  private version: string;
  private document: Document;
  private readonly disk: Disk;
  private timer: ReturnType<typeof setTimeout> | undefined;
  private saving = false;
  private waiting: NoteText | undefined;
  private readonly ours = new Set<string>();
  private running: Promise<void> = Promise.resolve();
  private closed = false;
  private lost = false;
  private readonly key: string;

  constructor(
    base: string,
    version: string,
    document: Document,
    disk: Disk,
    key = "",
  ) {
    this.base = base;
    this.version = version;
    this.document = document;
    this.disk = disk;
    this.key = key;
    open.add(this);
  }

  leftover(key: string): Resumed | undefined {
    if (key !== this.key || !this.closed || this.saving) {
      return undefined;
    }

    clearTimeout(this.timer);
    this.timer = undefined;

    return {
      base: this.base,
      version: this.version,
      text: this.document.text(),
    };
  }

  async finish() {
    for (let round = 0; round < rounds; round++) {
      clearTimeout(this.timer);
      this.timer = undefined;
      await this.running;
      if (this.document.text() === this.base) {
        this.forget();
        return true;
      }

      await this.save();
      if (this.lost) {
        return false;
      }
    }

    return this.document.text() === this.base;
  }

  edited() {
    clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      this.timer = undefined;
      void this.save();
    }, pause);
  }

  flush() {
    clearTimeout(this.timer);
    this.timer = undefined;
    void this.save();
  }

  close() {
    let text = this.document.text();
    this.document = {
      text: () => text,
      replace: (next) => {
        text = next;
      },
    };

    this.closed = true;
    this.flush();
  }

  loaded(text: string, version: string) {
    if (this.saving) {
      this.waiting = { text, version };
      return;
    }
    if (version === this.version || this.ours.has(version)) {
      return;
    }

    this.take(text, version);
  }

  private take(text: string, version: string) {
    this.ours.clear();
    const mine = this.document.text();
    const merged = merge(this.base, mine, text);
    if (merged.kept) {
      this.disk.kept();
    }
    if (merged.text !== mine) {
      this.document.replace(merged.text);
    }

    this.base = text;
    this.version = version;
    if (this.document.text() !== this.base) {
      this.edited();
    }
  }

  private save() {
    const text = this.document.text();
    if (this.saving || text === this.base) {
      this.forget();
      return this.running;
    }

    this.running = this.write(text);
    return this.running;
  }

  private forget() {
    if (
      this.closed &&
      !this.saving &&
      this.timer === undefined &&
      this.document.text() === this.base
    ) {
      open.delete(this);
    }
  }

  private async write(text: string) {
    this.saving = true;
    let failed = false;
    let current: NoteText | undefined;
    try {
      const written = await this.disk.write(text, this.version);
      if (written.kind === "saved") {
        this.base = text;
        this.version = written.version;
        this.ours.add(written.version);
        this.disk.saved(text, written.version);
      } else {
        current = await this.disk.read();
      }
    } catch (error) {
      failed = true;
      this.disk.failed(error);
    } finally {
      this.saving = false;
      this.lost = failed;
    }

    const waiting = this.waiting;
    this.waiting = undefined;
    if (current) {
      this.take(current.text, current.version);
    } else if (waiting) {
      this.loaded(waiting.text, waiting.version);
    } else if (!failed && this.document.text() !== this.base) {
      this.edited();
    }
    this.forget();
  }
}
