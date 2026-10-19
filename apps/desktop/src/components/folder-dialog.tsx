import { ArrowUpIcon, FolderIcon } from "@phosphor-icons/react";
import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@/components/ui/input-group";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Spinner } from "@/components/ui/spinner";
import { foldersQuery } from "@/lib/folders";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  open: boolean;
  start: string;
  onOpenChange: (open: boolean) => void;
  onChoose: (path: string) => void;
}

export function FolderDialog({
  machine,
  open,
  start,
  onOpenChange,
  onChoose,
}: Props) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Choose a folder</DialogTitle>
        </DialogHeader>
        <FolderBrowser machine={machine} start={start} onChoose={onChoose} />
      </DialogContent>
    </Dialog>
  );
}

interface BrowserProps {
  machine: Machine;
  start: string;
  onChoose: (path: string) => void;
}

function FolderBrowser({ machine, start, onChoose }: BrowserProps) {
  const [path, setPath] = useState(start);
  const { data, error, isFetching } = useQuery({
    ...foldersQuery(machine, path),
    placeholderData: keepPreviousData,
  });

  return (
    <>
      <div className="flex min-w-0 flex-col gap-2">
        <form
          onSubmit={(event) => {
            event.preventDefault();
            event.stopPropagation();
            const typed = new FormData(event.currentTarget).get("path");
            if (typeof typed === "string" && typed.trim()) {
              setPath(typed.trim());
            }
          }}
        >
          <InputGroup>
            <InputGroupAddon>
              <InputGroupButton
                size="icon-xs"
                aria-label="Up one folder"
                disabled={!data?.parent}
                onClick={() => {
                  if (data?.parent) {
                    setPath(data.parent);
                  }
                }}
              >
                <ArrowUpIcon />
              </InputGroupButton>
            </InputGroupAddon>
            <InputGroupInput
              key={data?.path}
              name="path"
              aria-label="Folder"
              defaultValue={data?.path ?? path}
              spellCheck={false}
            />
            {isFetching && (
              <InputGroupAddon align="inline-end">
                <Spinner aria-hidden />
              </InputGroupAddon>
            )}
          </InputGroup>
        </form>
        <ScrollArea className="h-64">
          <div className="flex flex-col">
            {data?.folders.map((folder) => (
              <Button
                key={folder.path}
                variant="ghost"
                size="sm"
                className="justify-start"
                onClick={() => {
                  setPath(folder.path);
                }}
              >
                <FolderIcon data-icon="inline-start" />
                <span className="truncate">{folder.name}</span>
              </Button>
            ))}
            {data?.folders.length === 0 && (
              <p className="p-2 type-body">No folders here</p>
            )}
          </div>
        </ScrollArea>
        {error && <p className="type-body">{error.message}</p>}
      </div>
      <DialogFooter>
        <DialogClose render={<Button variant="ghost" size="sm" />}>
          Cancel
        </DialogClose>
        <Button
          variant="secondary"
          size="sm"
          disabled={!data}
          onClick={() => {
            if (data) {
              onChoose(data.path);
            }
          }}
        >
          Use this folder
        </Button>
      </DialogFooter>
    </>
  );
}
