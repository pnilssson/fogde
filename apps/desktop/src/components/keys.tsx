import { cn } from "cn";

import { keys } from "@/lib/shortcuts";

export function Keys({
  letter,
  alt = false,
  className,
}: {
  letter: string;
  alt?: boolean;
  className?: string;
}) {
  return (
    <kbd
      className={cn(
        "font-sans type-caption tracking-normal whitespace-nowrap",
        className,
      )}
    >
      {keys(letter, alt)}
    </kbd>
  );
}
