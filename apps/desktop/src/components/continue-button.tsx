import { useState } from "react";

import { ActionButton } from "@/components/action-button";

interface Props {
  disabled?: boolean;
  onContinue: () => Promise<void>;
}

export function ContinueButton({ disabled = false, onContinue }: Props) {
  const [loading, setLoading] = useState(false);

  return (
    <ActionButton
      label="Continue"
      busy="Continuing…"
      pending={loading}
      disabled={disabled}
      onClick={() => {
        setLoading(true);
        void onContinue().finally(() => {
          setLoading(false);
        });
      }}
    />
  );
}
