import type { DockviewPanelApi } from "dockview-react";
import { createContext, type ReactNode } from "react";

import type { Machine } from "@/lib/machines";

interface TerminalProps {
  id: string;
  machine: Machine;
  panel: DockviewPanelApi;
}

export type RenderTerminal = (props: TerminalProps) => ReactNode;

export const TerminalSlot = createContext<RenderTerminal>(() => null);

interface NotesProps {
  machine: Machine;
  note: string | undefined;
  onOpen: (note: string) => void;
}

export type RenderNotes = (props: NotesProps) => ReactNode;

export const NotesSlot = createContext<RenderNotes>(() => null);

interface PaletteNotesProps {
  machine: Machine;
  onOpen: (note: string) => void;
}

export type RenderPaletteNotes = (props: PaletteNotesProps) => ReactNode;
