interface Props {
  label: string;
  value: string;
}

export function Labelled({ label, value }: Props) {
  return (
    <span className="flex min-w-0 items-baseline gap-2">
      <span className="shrink-0 type-caption text-muted-foreground">
        {label}
      </span>
      <span className="min-w-0 truncate type-body">{value}</span>
    </span>
  );
}
