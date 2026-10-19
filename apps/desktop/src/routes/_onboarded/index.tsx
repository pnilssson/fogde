import { createFileRoute } from "@tanstack/react-router";

import { TerminalTiles } from "@/features/terminals/components/terminal-tiles";

export const Route = createFileRoute("/_onboarded/")({
  component: TerminalTiles,
});
