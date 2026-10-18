import { openUrl } from "@tauri-apps/plugin-opener";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@/components/ui/input-group";
import { Spinner } from "@/components/ui/spinner";

interface Props {
  name: string;
  url: string;
  submitting: boolean;
  onSubmit: (code: string) => void;
  onCancel: () => void;
}

export function PasteCode({
  name,
  url,
  submitting,
  onSubmit,
  onCancel,
}: Props) {
  const [code, setCode] = useState("");

  return (
    <section className="flex w-full max-w-sm flex-col items-center gap-8">
      <h1 className="type-display">
        Paste the code from{" "}
        <button
          type="button"
          className="underline underline-offset-4"
          onClick={() => void openUrl(url)}
        >
          {name}
        </button>
      </h1>
      <form
        className="w-full"
        onSubmit={(event) => {
          event.preventDefault();
          if (code.trim() && !submitting) {
            onSubmit(code.trim());
          }
        }}
      >
        <InputGroup>
          <InputGroupInput
            autoFocus
            aria-label={`Code from ${name}`}
            placeholder="Paste the code"
            autoComplete="off"
            spellCheck={false}
            disabled={submitting}
            value={code}
            onChange={(event) => {
              setCode(event.target.value);
            }}
          />
          <InputGroupAddon align="inline-end">
            <InputGroupButton type="submit" aria-busy={submitting}>
              {submitting && <Spinner aria-hidden data-icon="inline-start" />}
              {submitting ? "Connecting…" : "Connect"}
            </InputGroupButton>
          </InputGroupAddon>
        </InputGroup>
      </form>
      <Button variant="ghost" size="sm" onClick={onCancel}>
        Cancel
      </Button>
    </section>
  );
}
