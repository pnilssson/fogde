interface Props {
  label: string;
  value: string;
  under?: boolean;
}

export function Labelled({ label, value, under = false }: Props) {
  const note = (
    <span className="shrink-0 type-caption text-muted-foreground">{label}</span>
  );

  return (
    <span className="flex min-w-0 items-baseline gap-2">
      {!under && note}
      <span className="min-w-0 truncate type-body">{value}</span>
      {under && note}
    </span>
  );
}
