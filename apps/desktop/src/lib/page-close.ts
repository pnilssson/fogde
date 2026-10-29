import { createContext } from "react";

export const PageClose = createContext<(() => void) | undefined>(undefined);
