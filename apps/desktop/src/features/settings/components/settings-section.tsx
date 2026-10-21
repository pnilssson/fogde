import type { ReactNode } from "react";

interface SectionProps {
  id: string;
  title: string;
  action?: ReactNode;
  children: ReactNode;
}

export function SettingsSection({ id, title, action, children }: SectionProps) {
  return (
    <section className="flex flex-col gap-1" aria-labelledby={id}>
      <div className="flex h-8 items-center gap-2">
        <h2 id={id} className="min-w-0 flex-1 truncate type-title">
          {title}
        </h2>
        {action}
      </div>
      {children}
    </section>
  );
}

interface RowProps {
  label: ReactNode;
  leading?: ReactNode;
  labelled?: boolean;
  children: ReactNode;
}

export function SettingsRow({
  label,
  leading,
  labelled = false,
  children,
}: RowProps) {
  const Root = labelled ? "label" : "div";

  return (
    <Root className="flex h-8 items-center gap-2">
      {leading}
      <span className="min-w-0 flex-1 truncate type-body">{label}</span>
      {children}
    </Root>
  );
}
