import { useEffect, useState } from "react";

import type { Scroll } from "@/features/terminals/lib/scrollbar";
import type { Client } from "@/lib/client";

export function useScrolled(client: Client, id: string) {
  const [scroll, setScroll] = useState<Scroll>();

  useEffect(
    () =>
      client.listen("terminals.scrolled", (scrolled) => {
        if (scrolled.id === id) {
          setScroll(scrolled);
        }
      }),
    [client, id],
  );

  return scroll;
}
