import { useState } from "react";

import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@/components/ui/input-group";
import { Spinner } from "@/components/ui/spinner";

interface Props {
  starting: boolean;
  onSubmit: (host: string) => void;
}

export function EnterpriseForm({ starting, onSubmit }: Props) {
  const [host, setHost] = useState("");

  return (
    <form
      className="w-full"
      onSubmit={(event) => {
        event.preventDefault();
        if (host.trim() && !starting) {
          onSubmit(host);
        }
      }}
    >
      <InputGroup>
        <InputGroupInput
          autoFocus
          aria-label="GitHub Enterprise address"
          placeholder="company.ghe.com"
          spellCheck={false}
          value={host}
          onChange={(event) => {
            setHost(event.target.value);
          }}
        />
        <InputGroupAddon align="inline-end">
          <InputGroupButton type="submit" aria-busy={starting}>
            {starting && <Spinner aria-hidden data-icon="inline-start" />}
            {starting ? "Connecting…" : "Connect"}
          </InputGroupButton>
        </InputGroupAddon>
      </InputGroup>
    </form>
  );
}
