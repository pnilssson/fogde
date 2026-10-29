import { type CSSProperties, type PointerEvent, useRef, useState } from "react";

import {
  dragged,
  place,
  type Scroll,
  share,
} from "@/features/terminals/lib/scrollbar";

interface Props {
  scroll: Scroll | undefined;
  onSeek: (offset: number) => void;
}

interface Grab {
  y: number;
  offset: number;
  travel: number;
}

export function TerminalScrollbar({ scroll, onSeek }: Props) {
  const track = useRef<HTMLDivElement>(null);
  const grab = useRef<Grab>(undefined);
  const frame = useRef(0);
  const [offset, setOffset] = useState<number>();

  if (!scroll || scroll.max === 0) {
    return null;
  }

  const start = (event: PointerEvent<HTMLDivElement>) => {
    const bar = track.current;
    if (!bar) {
      return;
    }

    event.currentTarget.setPointerCapture(event.pointerId);
    grab.current = {
      y: event.clientY,
      offset: scroll.offset,
      travel:
        bar.getBoundingClientRect().height -
        event.currentTarget.getBoundingClientRect().height,
    };
    setOffset(scroll.offset);
  };

  const move = (event: PointerEvent<HTMLDivElement>) => {
    const origin = grab.current;
    if (!origin) {
      return;
    }

    const next = dragged(
      scroll,
      origin.offset,
      event.clientY - origin.y,
      origin.travel,
    );
    setOffset(next);
    cancelAnimationFrame(frame.current);
    frame.current = requestAnimationFrame(() => {
      onSeek(next);
    });
  };

  const end = () => {
    grab.current = undefined;
    setOffset(undefined);
  };

  const style = {
    "--size": `max(2rem, ${String(share(scroll) * 100)}%)`,
    "--at": place(scroll, offset ?? scroll.offset),
  } as CSSProperties;

  return (
    <div
      ref={track}
      aria-hidden
      className="pointer-events-none absolute top-0 right-0 bottom-2 w-2"
      style={style}
    >
      <div
        className="terminal-thumb pointer-events-auto"
        data-dragging={offset === undefined ? undefined : ""}
        onPointerDown={start}
        onPointerMove={move}
        onPointerUp={end}
        onPointerCancel={end}
      />
    </div>
  );
}
