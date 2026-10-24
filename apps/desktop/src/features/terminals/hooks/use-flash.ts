import { useCallback, useEffect, useRef, useState } from "react";

export function useFlash(duration: number) {
  const timer = useRef(0);
  const [on, setOn] = useState(false);

  useEffect(
    () => () => {
      window.clearTimeout(timer.current);
    },
    [],
  );

  const flash = useCallback(() => {
    setOn(true);
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => {
      setOn(false);
    }, duration);
  }, [duration]);

  return [on, flash] as const;
}
