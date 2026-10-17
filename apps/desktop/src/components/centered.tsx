import type { ReactNode } from "react";

interface Props {
  children: ReactNode;
}

export function Centered({ children }: Props) {
  return (
    <main className="flex h-full items-center justify-center p-8">
      {children}
    </main>
  );
}
