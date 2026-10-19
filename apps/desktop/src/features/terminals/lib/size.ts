import type { FitAddon } from "@xterm/addon-fit";

import type { Size } from "@/generated/protocol";

export function measure(fit: FitAddon): Size | undefined {
  const size = fit.proposeDimensions();
  if (!size || !(size.cols >= 1 && size.rows >= 1)) {
    return undefined;
  }

  return { cols: size.cols, rows: size.rows };
}
