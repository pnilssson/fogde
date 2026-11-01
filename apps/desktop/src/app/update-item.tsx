import { ArrowCircleUpIcon } from "@phosphor-icons/react";
import { useMutation, useQuery } from "@tanstack/react-query";
import type { Update } from "@tauri-apps/plugin-updater";
import { useEffect } from "react";

import { SidebarMenuButton, SidebarMenuItem } from "@/components/ui/sidebar";
import { Spinner } from "@/components/ui/spinner";
import { settle } from "@/features/notes/hooks/use-save-on-close";
import { report } from "@/lib/report";
import { restart, updateQuery } from "@/lib/update";

let warned = false;

export function UpdateItem() {
  const { data: update, error } = useQuery(updateQuery);
  const install = useMutation({
    mutationFn: async (next: Update) => {
      if (!(await settle())) {
        throw new Error("A note could not be saved; try again once it is.");
      }

      await restart(next);
    },
    onError: (error) => {
      report("Could not update", error);
    },
  });

  useEffect(() => {
    if (!error || warned) {
      return;
    }

    warned = true;
    report("Could not check for updates", error);
  }, [error]);

  if (!update) {
    return null;
  }

  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        className="text-muted-foreground"
        disabled={install.isPending}
        onClick={() => {
          install.mutate(update);
        }}
      >
        {install.isPending ? <Spinner aria-hidden /> : <ArrowCircleUpIcon />}
        <span className="min-w-0 flex-1 truncate">Update ready</span>
        <span className="type-caption tabular-nums">{update.version}</span>
      </SidebarMenuButton>
    </SidebarMenuItem>
  );
}
