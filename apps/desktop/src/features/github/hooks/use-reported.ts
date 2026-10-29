import { useEffect, useRef } from "react";

import { report } from "@/lib/report";

export function useReported(title: string, error: Error | null) {
  const last = useRef<string>(undefined);

  useEffect(() => {
    const message = error?.message;
    if (message && message !== last.current) {
      report(title, error);
    }

    last.current = message;
  }, [title, error]);
}
