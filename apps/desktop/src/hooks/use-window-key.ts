import { useEffect, useEffectEvent } from "react";

type Choose = (event: KeyboardEvent) => (() => void) | undefined;

export function useWindowKey(choose: Choose, capture = true) {
  const action = useEffectEvent(choose);

  useEffect(() => {
    const down = (event: KeyboardEvent) => {
      if (event.defaultPrevented) {
        return;
      }

      const run = action(event);
      if (!run) {
        return;
      }

      event.preventDefault();
      event.stopPropagation();
      run();
    };

    window.addEventListener("keydown", down, { capture });

    return () => {
      window.removeEventListener("keydown", down, { capture });
    };
  }, [capture]);
}
