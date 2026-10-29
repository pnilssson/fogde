import type { ComponentProps, ReactNode } from "react";

import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";

interface Props extends Omit<ComponentProps<typeof Button>, "children"> {
  label: string;
  busy: string;
  pending: boolean;
  icon?: ReactNode;
}

export function ActionButton({
  label,
  busy,
  pending,
  icon,
  disabled,
  ...props
}: Props) {
  return (
    <Button
      size="sm"
      variant="secondary"
      disabled={pending || disabled}
      aria-busy={pending}
      {...props}
    >
      {pending ? <Spinner aria-hidden data-icon="inline-start" /> : icon}
      {pending ? busy : label}
    </Button>
  );
}
