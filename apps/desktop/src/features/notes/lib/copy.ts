import { flashCopied } from "@/lib/copied";
import { report } from "@/lib/report";

export const copiedPath = "notes";

export function copyPath(path: string) {
  navigator.clipboard.writeText(path).then(
    () => {
      flashCopied(copiedPath);
    },
    (error: unknown) => {
      report("Could not copy the path", error);
    },
  );
}
