import { useEffect, useRef, useState } from "react";

import type { Range } from "@/features/monitor/lib/chart";

const FRAME = 32;
const STILL_FRAME = 1000;
const EASE = 0.15;

export interface Live {
  now: number;
  value: number;
  range: Range;
}

export function useLive(value: number, range: Range): Live {
  const target = useRef({ value, range });
  const [live, setLive] = useState<Live>(() => ({
    now: Date.now(),
    value,
    range,
  }));

  useEffect(() => {
    target.current = { value, range };
  }, [value, range]);

  useEffect(() => {
    const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
    const gap = still ? STILL_FRAME : FRAME;
    const ease = still ? 1 : EASE;
    let frame = 0;
    let last = 0;
    const tick = (time: number) => {
      frame = requestAnimationFrame(tick);
      if (time - last < gap) {
        return;
      }

      last = time;
      setLive((current) => step(current, target.current, ease));
    };
    frame = requestAnimationFrame(tick);

    return () => {
      cancelAnimationFrame(frame);
    };
  }, []);

  return live;
}

function step(
  current: Live,
  target: { value: number; range: Range },
  ease: number,
): Live {
  const [low, high] = current.range;
  const [wantLow, wantHigh] = target.range;

  return {
    now: Date.now(),
    value: current.value + (target.value - current.value) * ease,
    range: [
      wantLow < low ? wantLow : low + (wantLow - low) * ease,
      wantHigh > high ? wantHigh : high + (wantHigh - high) * ease,
    ],
  };
}
