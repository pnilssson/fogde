import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

import { closeActive } from "@/features/workspaces/lib/closing";
import { moveTile } from "@/features/workspaces/lib/drag";
import { togglePalette } from "@/features/workspaces/lib/palette";
import type { useGroups } from "@/features/workspaces/hooks/use-terminals";
import {
  findWorkspace,
  type View,
  viewFolder,
} from "@/features/workspaces/lib/selection";
import { focusTile } from "@/features/workspaces/lib/tile-focus";
import { openStart } from "@/features/workspaces/lib/opening";
import { revealNext } from "@/features/workspaces/lib/tiles";
import { toggleSettings } from "@/lib/settings-open";
import { arrowSide, pressed, shortcuts, tileNumber } from "@/lib/shortcuts";

type Groups = ReturnType<typeof useGroups>;

interface Context {
  groups: Groups;
  workspace: View | undefined;
}

export function useShortcuts(groups: Groups, workspace: View | undefined) {
  const current = useRef<Context>({ groups, workspace });

  useEffect(() => {
    current.current = { groups, workspace };
  }, [groups, workspace]);

  useEffect(() => {
    const down = (event: KeyboardEvent) => {
      const action = actionFor(event, current.current);
      if (!action) {
        return;
      }

      event.preventDefault();
      event.stopPropagation();
      action();
    };

    window.addEventListener("keydown", down, { capture: true });
    const stopMenu = listen("close-tile", () => {
      if (!inDialog(document.activeElement)) {
        closeActive();
      }
    });

    return () => {
      window.removeEventListener("keydown", down, { capture: true });
      void stopMenu.then((stop) => {
        stop();
      });
    };
  }, []);
}

function actionFor(event: KeyboardEvent, { groups, workspace }: Context) {
  if (pressed(event, shortcuts.palette)) {
    return togglePalette;
  }
  if (pressed(event, shortcuts.settings)) {
    return toggleSettings;
  }
  if (inDialog(event.target)) {
    return undefined;
  }

  if (pressed(event, shortcuts.attention)) {
    return () => {
      revealNext(groups);
    };
  }

  if (pressed(event, shortcuts.close)) {
    return closeActive;
  }

  if (pressed(event, shortcuts.tile)) {
    const found = findWorkspace(groups, workspace);

    return found && !found.workspace.missing
      ? () => {
          openStart(viewFolder(found.workspace, workspace), true);
        }
      : undefined;
  }

  const side = arrowSide(event);
  if (side) {
    return () => {
      moveTile(side);
    };
  }

  const number = tileNumber(event);

  return number
    ? () => {
        focusTile(number);
      }
    : undefined;
}

function inDialog(target: EventTarget | null) {
  return (
    target instanceof Element &&
    target.closest('[role="dialog"], [role="alertdialog"]') !== null
  );
}
