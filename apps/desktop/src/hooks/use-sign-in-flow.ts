import { openUrl } from "@tauri-apps/plugin-opener";
import { useRef, useState } from "react";

import { report } from "@/lib/report";
import { trusted, UntrustedLink } from "@/lib/sign-in";

export interface Flow<K, P> {
  start: (key: K) => Promise<P | undefined>;
  link: (
    key: K,
    prompt: P,
  ) => { url: string; domains: readonly string[] } | undefined;
  waits: (prompt: P) => boolean;
  finish: (key: K, code: string | null) => Promise<unknown>;
  cancel: (key: K) => Promise<unknown>;
  refresh: () => Promise<unknown>;
  failed: (key: K) => string;
  stuck: (key: K) => string;
}

interface Active<K, P> {
  key: K;
  prompt?: P;
  submitting: boolean;
}

export function useSignInFlow<K, P>(flow: Flow<K, P>) {
  const [active, setActive] = useState<Active<K, P>>();
  const attempt = useRef(0);

  const current = (id: number) => attempt.current === id;
  const end = (id: number) => {
    if (current(id)) {
      attempt.current += 1;
      setActive(undefined);
    }
  };
  const done = async (id: number) => {
    if (current(id)) {
      await flow.refresh();
    }

    end(id);
  };
  const fail = (id: number, key: K, error: unknown) => {
    if (current(id)) {
      report(flow.failed(key), error);
    }

    end(id);
  };
  const finish = (id: number, key: K, code: string | null) => {
    setActive((shown) => shown && { ...shown, submitting: true });
    flow.finish(key, code).then(
      () => done(id),
      (error: unknown) => {
        fail(id, key, error);
      },
    );
  };

  const begin = async (key: K) => {
    attempt.current += 1;
    const id = attempt.current;
    setActive({ key, submitting: false });

    let prompt: P | undefined;
    try {
      prompt = await flow.start(key);
    } catch (error) {
      fail(id, key, error);
      return;
    }

    if (!current(id)) {
      return;
    }
    if (prompt === undefined) {
      await done(id);
      return;
    }

    const link = flow.link(key, prompt);
    if (link && !trusted(link.url, link.domains)) {
      void flow.cancel(key).catch(() => undefined);
      fail(id, key, new UntrustedLink(link.url));
      return;
    }

    setActive({ key, prompt, submitting: false });
    if (link) {
      void openUrl(link.url);
    }
    if (flow.waits(prompt)) {
      finish(id, key, null);
    }
  };

  return {
    active,
    submitting: active?.submitting ?? false,
    begin: (key: K) => {
      void begin(key);
    },
    paste: (code: string) => {
      if (active) {
        finish(attempt.current, active.key, code);
      }
    },
    cancel: () => {
      if (!active) {
        return;
      }

      end(attempt.current);
      flow
        .cancel(active.key)
        .catch((error: unknown) => {
          report(flow.stuck(active.key), error);
        })
        .finally(() => void flow.refresh());
    },
  };
}
