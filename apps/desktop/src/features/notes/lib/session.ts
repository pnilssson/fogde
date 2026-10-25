import { merge } from "@/features/notes/lib/merge";
import type { Written } from "@/generated/protocol";

const pause = 600;

export interface Disk {
  write: (text: string, version: string) => Promise<Written>;
  saved: (text: string, version: string) => void;
  reload: () => void;
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
  private readonly document: Document;
  private readonly disk: Disk;
  private timer: ReturnType<typeof setTimeout> | undefined;
  private saving = false;
  private waiting: { text: string; version: string } | undefined;
  private readonly ours = new Set<string>();

  constructor(base: string, version: string, document: Document, disk: Disk) {
    this.base = base;
    this.version = version;
    this.document = document;
    this.disk = disk;
  }

  edited() {
    clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      void this.save();
    }, pause);
  }

  flush() {
    clearTimeout(this.timer);
    void this.save();
  }

  loaded(text: string, version: string) {
    if (this.saving) {
      this.waiting = { text, version };
      return;
    }
    if (version === this.version || this.ours.has(version)) {
      return;
    }

    this.ours.clear();
    const mine = this.document.text();
    const merged = merge(this.base, mine, text);
    if (merged === undefined) {
      this.disk.kept();
    } else if (merged !== mine) {
      this.document.replace(merged);
    }

    this.base = text;
    this.version = version;
    if (this.document.text() !== this.base) {
      this.edited();
    }
  }

  private async save() {
    const text = this.document.text();
    if (this.saving || text === this.base) {
      return;
    }

    this.saving = true;
    let failed = false;
    try {
      const written = await this.disk.write(text, this.version);
      if (written.kind === "saved") {
        this.base = text;
        this.version = written.version;
        this.ours.add(written.version);
        this.disk.saved(text, written.version);
      } else {
        this.disk.reload();
      }
    } catch (error) {
      failed = true;
      this.disk.failed(error);
    } finally {
      this.saving = false;
    }

    const waiting = this.waiting;
    this.waiting = undefined;
    if (waiting) {
      this.loaded(waiting.text, waiting.version);
    } else if (!failed && this.document.text() !== this.base) {
      this.edited();
    }
  }
}
