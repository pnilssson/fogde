const started = 0.25;

interface Props {
  total: number;
  position: number;
}

function fill(index: number, position: number) {
  if (index < position) {
    return 1;
  }

  return index === position ? started : 0;
}

export function StepBars({ total, position }: Props) {
  return (
    <div className="flex items-center gap-2" aria-hidden>
      {Array.from({ length: total }, (_, index) => (
        <span
          key={index}
          className="h-1 w-6 overflow-hidden rounded-full bg-border"
        >
          <span
            className="block h-full origin-left rounded-full bg-foreground transition-transform duration-200 ease-out motion-reduce:transition-none"
            style={{ transform: `scaleX(${String(fill(index, position))})` }}
          />
        </span>
      ))}
    </div>
  );
}
