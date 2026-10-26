import { useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";

import { MonitorView } from "@/features/monitor/components/monitor-view";
import { NotesView } from "@/features/notes/components/notes-view";
import { TerminalView } from "@/features/terminals/components/terminal-view";
import { workspacesQuery } from "@/features/workspaces/api/workspaces";
import { TerminalTiles } from "@/features/workspaces/components/terminal-tiles";

export const Route = createFileRoute("/_onboarded/")({
  component: function TilesRoute() {
    const queries = useQueryClient();

    return (
      <TerminalTiles
        notes={(props) => <NotesView {...props} />}
        monitor={(props) => <MonitorView {...props} />}
        terminal={(props) => (
          <TerminalView
            {...props}
            onChange={() => {
              void queries.invalidateQueries(workspacesQuery(props.machine));
            }}
          />
        )}
      />
    );
  },
});
