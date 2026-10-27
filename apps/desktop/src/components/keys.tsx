import { Kbd } from "@/components/ui/kbd";
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
  return <Kbd className={className}>{keys(letter, alt)}</Kbd>;
}
