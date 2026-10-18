import { Link } from "@tanstack/react-router";

import { Button } from "@/components/ui/button";

export function Home() {
  return (
    <section className="flex max-w-sm flex-col items-start gap-2">
      <h1 className="type-display">Welcome to Fogde</h1>
      <p className="type-body">Your workspace is ready.</p>
      {import.meta.env.DEV && (
        <Button
          variant="secondary"
          size="sm"
          nativeButton={false}
          render={<Link to="/onboarding/github" />}
        >
          Open onboarding
        </Button>
      )}
    </section>
  );
}
