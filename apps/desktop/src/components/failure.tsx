import { useRouter, type ErrorComponentProps } from "@tanstack/react-router";

import { Centered } from "@/components/centered";
import { Button } from "@/components/ui/button";

export function Failure({ error }: ErrorComponentProps) {
  const router = useRouter();

  return (
    <Centered>
      <section className="flex max-w-sm flex-col items-start gap-4">
        <h1 className="type-title">Something went wrong</h1>
        <p className="type-body">
          {error instanceof Error ? error.message : String(error)}
        </p>
        <Button onClick={() => void router.invalidate()}>Try again</Button>
      </section>
    </Centered>
  );
}
