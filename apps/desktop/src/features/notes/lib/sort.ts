import { z } from "zod";
import { useStore } from "zustand";
import { persist } from "zustand/middleware";
import { createStore } from "zustand/vanilla";

import { type Order, orders } from "@/features/notes/lib/tree";

const saved = z.object({ order: z.enum(orders) });

const store = createStore<{ order: Order }>()(
  persist((): { order: Order } => ({ order: "name-asc" }), {
    name: "fogde.notes",
    merge: (persisted, current) => ({
      ...current,
      ...saved.safeParse(persisted).data,
    }),
  }),
);

export function useOrder() {
  return useStore(store, ({ order }) => order);
}

export function setOrder(order: Order) {
  store.setState({ order });
}
