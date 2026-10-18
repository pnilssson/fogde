import { cubicBezier, type MotionProps, type Transition } from "motion/react";

let curve: ((progress: number) => number) | undefined;

function easeOut(progress: number) {
  curve ??= cubicBezier(
    ...(getComputedStyle(document.documentElement)
      .getPropertyValue("--ease-out")
      .match(/-?[\d.]+/g)
      ?.map(Number) as [number, number, number, number]),
  );

  return curve(progress);
}

const enter: Transition = { duration: 0.22, ease: easeOut };

const leave: Transition = { duration: 0.15, ease: easeOut };

export const swap = {
  initial: { opacity: 0, y: 8 },
  animate: { opacity: 1, y: 0, transition: enter },
  exit: { opacity: 0, y: -8, transition: leave },
} satisfies MotionProps;

export const expand = {
  initial: { opacity: 0, height: 0, overflow: "hidden" },
  animate: {
    opacity: 1,
    height: "auto",
    transition: enter,
    transitionEnd: { overflow: "visible" },
  },
  exit: { opacity: 0, height: 0, overflow: "hidden", transition: leave },
} satisfies MotionProps;
