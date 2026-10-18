import { useState } from "react";

import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";

interface Props {
  disabled?: boolean;
  onContinue: () => Promise<void>;
}

export function ContinueButton({ disabled = false, onContinue }: Props) {
  const [loading, setLoading] = useState(false);

  return (
    <Button
      variant="secondary"
      size="sm"
      aria-busy={loading}
      disabled={disabled || loading}
      onClick={() => {
        setLoading(true);
        void onContinue().finally(() => {
          setLoading(false);
        });
      }}
    >
      {loading && <Spinner aria-hidden data-icon="inline-start" />}
      {loading ? "Loading…" : "Continue"}
    </Button>
  );
}
