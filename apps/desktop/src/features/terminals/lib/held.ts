import { ordered, type Range } from "@/features/terminals/lib/range";
import type { Client } from "@/lib/client";

interface Held {
  range: Range;
  text: string;
}

export function holdText(
  client: Client,
  id: string,
  current: () => Range | undefined,
  lost: () => void,
) {
  let held: Held | undefined;
  let checking = false;
  let again = false;

  const fetch = (range: Range) => {
    const [start, end] = ordered(range);

    return client.call("terminals.selection", { id, start, end });
  };

  const read = async (range: Range) => {
    if (held?.range === range) {
      return held.text;
    }

    const text = await fetch(range);
    if (current() === range) {
      held = { range, text };
    }

    return text;
  };

  const check = () => {
    const range = current();
    if (!held || held.range !== range) {
      return;
    }
    if (checking) {
      again = true;
      return;
    }

    checking = true;
    const { text } = held;
    fetch(range)
      .then(
        (value) => {
          if (current() === range && value !== text) {
            lost();
          }
        },
        () => undefined,
      )
      .finally(() => {
        checking = false;
        if (again) {
          again = false;
          check();
        }
      });
  };

  return { read, check };
}
