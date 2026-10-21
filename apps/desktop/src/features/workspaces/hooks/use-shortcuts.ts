import { useEffect, useRef } from "react";

import { togglePalette } from "@/features/workspaces/lib/palette";
import type { useGroups } from "@/features/workspaces/hooks/use-terminals";
import { focusTile, revealNext } from "@/features/workspaces/lib/tiles";
import { pressed, shortcuts, tileNumber } from "@/lib/shortcuts";

type Groups = ReturnType<typeof useGroups>;

export function useShortcuts(groups: Groups) {
  const current = useRef(groups);

  useEffect(() => {
    current.current = groups;
  }, [groups]);

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

    return () => {
      window.removeEventListener("keydown", down, { capture: true });
    };
  }, []);
}

function actionFor(event: KeyboardEvent, groups: Groups) {
  if (pressed(event, shortcuts.palette)) {
    return togglePalette;
  }
  if (inDialog(event.target)) {
    return undefined;
  }

  if (pressed(event, shortcuts.attention)) {
    return () => {
      revealNext(groups);
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
