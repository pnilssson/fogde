import { Kbd, KbdGroup } from "@/components/ui/kbd";
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
    <KbdGroup className={className}>
      {keys(letter, alt).map((key) => (
        <Kbd key={key}>{key}</Kbd>
      ))}
    </KbdGroup>
  );
}
