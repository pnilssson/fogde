import { CheckIcon, CopyIcon } from "@phosphor-icons/react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";

interface Props {
  code: string;
  url: string;
}

export function DeviceCode({ code, url }: Props) {
  const [copied, setCopied] = useState(false);

  const copy = () => {
    void navigator.clipboard.writeText(code).then(() => {
      setCopied(true);
    });
  };

  return (
    <section className="flex flex-col items-center gap-8">
      <h1 className="type-display">
        Enter this code on{" "}
        <button
          type="button"
          className="underline underline-offset-4"
          onClick={() => void openUrl(url)}
        >
          GitHub
        </button>
      </h1>
      <div className="flex items-center gap-2">
        <span className="type-code-display">{code}</span>
        <Button
          variant="ghost"
          size="icon"
          aria-label={copied ? "Copied" : "Copy code"}
          onClick={copy}
        >
          {copied ? <CheckIcon /> : <CopyIcon />}
        </Button>
      </div>
      <p className="flex items-center gap-2 type-body">
        <Spinner aria-hidden />
        Waiting for approval
      </p>
    </section>
  );
}
