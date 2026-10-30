import { createContext, useContext } from "react";

type Open = (path: string, near?: string) => void;

export const OpenCheckout = createContext<Open>(() => undefined);

export function useOpenCheckout() {
  return useContext(OpenCheckout);
}
