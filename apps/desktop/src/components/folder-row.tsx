import { FolderIcon, TrashIcon } from "@phosphor-icons/react";
import { useMutation, useQueryClient } from "@tanstack/react-query";

import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { toast } from "@/components/ui/toast";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { Folder } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  folder: Folder;
  onOpen: () => void;
}

export function FolderRow({ machine, folder, onOpen }: Props) {
  const queries = useQueryClient();
  const remove = useMutation({
    mutationFn: () => machine.client.call("folders.remove", folder.path),
    onSuccess: () =>
      queries.invalidateQueries({ queryKey: [machine.id, "folders.list"] }),
    onError: (error) => {
      toast.add({
        type: "error",
        title: `Could not delete ${folder.name}`,
        description: error.message,
      });
    },
  });

  return (
    <div className="group/folder relative">
      <Button
        variant="ghost"
        size="sm"
        className="w-full justify-start pr-8 group-hover/folder:bg-hover hover:bg-hover dark:hover:bg-hover"
        onClick={onOpen}
      >
        <FolderIcon data-icon="inline-start" />
        <span className="truncate">{folder.name}</span>
      </Button>
      <Tooltip>
        <TooltipTrigger
          render={
            <Button
              variant="ghost"
              size="icon-xs"
              aria-label={`Delete ${folder.name}`}
              className="absolute top-1/2 right-2 -translate-y-1/2 text-muted-foreground"
              disabled={remove.isPending}
              aria-busy={remove.isPending}
              onClick={() => {
                remove.mutate();
              }}
            />
          }
        >
          {remove.isPending ? <Spinner aria-hidden /> : <TrashIcon />}
        </TooltipTrigger>
        <TooltipContent>Delete</TooltipContent>
      </Tooltip>
    </div>
  );
}
