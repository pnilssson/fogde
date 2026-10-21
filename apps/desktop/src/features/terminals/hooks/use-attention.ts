import { focusManager, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

import { applyState } from "@/features/terminals/api/workspaces";
import { needsYou } from "@/features/terminals/lib/status";
import { isShown, selectWorkspace, show } from "@/features/terminals/lib/tiles";
import type { Status } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";
import { bringForward, notificationsQuery, notify } from "@/lib/notifications";
import { chime } from "@/lib/sound";

const messages: Partial<Record<Status, string>> = {
  blocked: "Needs your input in",
  done: "Finished in",
};

export function useAttention(machines: Machine[]) {
  const queries = useQueryClient();

  useEffect(() => {
    const stops = machines.map((machine) =>
      machine.client.listen("terminals.state", (state) => {
        const applied = applyState(queries, machine, state);
        const message = messages[state.status];
        if (!applied || !message || needsYou(applied.before.status)) {
          return;
        }

        const terminal = { machine: machine.id, id: state.id };
        if (focusManager.isFocused() && isShown(terminal)) {
          return;
        }

        void queries
          .query(notificationsQuery)
          .then(async ({ enabled, sound }) => {
            if (sound) {
              void chime().catch(() => undefined);
            }

            if (enabled) {
              await notify({
                title: state.name,
                body: `${message} ${applied.workspace.name}`,
                onClick: () => {
                  void bringForward();
                  selectWorkspace({
                    machine: machine.id,
                    id: applied.workspace.id,
                  });
                  show(terminal);
                },
              });
            }
          })
          .catch(() => undefined);
      }),
    );

    return () => {
      for (const stop of stops) {
        stop();
      }
    };
  }, [machines, queries]);
}
