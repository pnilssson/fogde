import { ArrowCircleUpIcon } from "@phosphor-icons/react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useEffect } from "react";

import { Button } from "@/components/ui/button";
import { SidebarFooter } from "@/components/ui/sidebar";
import { Spinner } from "@/components/ui/spinner";
import { toast } from "@/components/ui/toast";
import { restart, updateQuery } from "@/lib/update";

let warned = false;

function message(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function UpdateButton() {
  const { data: update, error } = useQuery(updateQuery);
  const install = useMutation({
    mutationFn: restart,
    onError: (error) => {
      toast.add({
        type: "error",
        title: "Could not update Fogde",
        description: message(error),
      });
    },
  });

  useEffect(() => {
    if (!error || warned) {
      return;
    }

    warned = true;
    toast.add({
      type: "error",
      title: "Could not check for updates",
      description: message(error),
    });
  }, [error]);

  if (!update) {
    return null;
  }

  return (
    <SidebarFooter>
      <Button
        variant="secondary"
        className="w-full justify-start"
        disabled={install.isPending}
        onClick={() => {
          install.mutate(update);
        }}
      >
        {install.isPending ? (
          <Spinner aria-hidden data-icon="inline-start" />
        ) : (
          <ArrowCircleUpIcon data-icon="inline-start" />
        )}
        Update ready
        <span className="ml-auto type-caption text-muted-foreground tabular-nums">
          {update.version}
        </span>
      </Button>
    </SidebarFooter>
  );
}
