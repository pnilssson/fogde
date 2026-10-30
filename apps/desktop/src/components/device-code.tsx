import { CheckIcon, CopyIcon, InfoIcon } from "@phosphor-icons/react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

interface Props {
  code: string;
  url: string;
  site: string;
  hint?: string;
  onCancel?: () => void;
}

export function DeviceCode({ code, url, site, hint, onCancel }: Props) {
  const [copied, setCopied] = useState(false);

  const copy = () => {
    void navigator.clipboard.writeText(code).then(() => {
      setCopied(true);
    });
  };

  return (
    <section className="flex flex-col items-center gap-8">
      <div className="flex items-center gap-2">
        <h1 className="type-display">
          Enter this code on{" "}
          <button
            type="button"
            className="underline underline-offset-4"
            onClick={() => void openUrl(url)}
          >
            {site}
          </button>
        </h1>
        {hint && (
          <Tooltip>
            <TooltipTrigger
              render={
                <Button
                  variant="ghost"
                  size="icon-xs"
                  aria-label="More information"
                />
              }
            >
              <InfoIcon />
            </TooltipTrigger>
            <TooltipContent>{hint}</TooltipContent>
          </Tooltip>
        )}
      </div>
      <div className="flex items-center gap-2">
        <span className="font-mono type-display tabular-nums">{code}</span>
        <Button
          variant="ghost"
          size="icon-sm"
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
      {onCancel && (
        <Button variant="ghost" size="sm" onClick={onCancel}>
          Cancel
        </Button>
      )}
    </section>
  );
}
