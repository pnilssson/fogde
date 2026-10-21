import { Kbd, KbdGroup } from "@/components/ui/kbd";
import { keys } from "@/lib/shortcuts";

export function Keys({
  letter,
  className,
}: {
  letter: string;
  className?: string;
}) {
  return (
    <KbdGroup className={className}>
      {keys(letter).map((key) => (
        <Kbd key={key}>{key}</Kbd>
      ))}
    </KbdGroup>
  );
}
