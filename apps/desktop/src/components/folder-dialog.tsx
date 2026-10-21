import { ArrowUpIcon } from "@phosphor-icons/react";
import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { useState } from "react";

import { FolderRow } from "@/components/folder-row";
import { NewFolder } from "@/components/new-folder";
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

interface Props extends BrowserProps {
  open: boolean;
  title: string;
  onOpenChange: (open: boolean) => void;
}

export function FolderDialog({ open, title, onOpenChange, ...browser }: Props) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
        </DialogHeader>
        <FolderBrowser {...browser} />
      </DialogContent>
    </Dialog>
  );
}

interface BrowserProps {
  machine: Machine;
  start: string;
  action: string;
  busy?: boolean | undefined;
  onChoose: (path: string) => void;
}

function FolderBrowser({
  machine,
  start,
  action,
  busy = false,
  onChoose,
}: BrowserProps) {
  const [path, setPath] = useState(start);
  const { data, error, isFetching } = useQuery({
    ...foldersQuery(machine, path),
    placeholderData: keepPreviousData,
  });
  const [naming, setNaming] = useState(false);

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
        {error && <p className="type-body">{error.message}</p>}
        {naming && data && (
          <NewFolder
            machine={machine}
            parent={data.path}
            onCreated={(created) => {
              setNaming(false);
              setPath(created);
            }}
          />
        )}
        <ScrollArea className="h-64">
          <div className="flex flex-col">
            {data?.folders.map((folder) => (
              <FolderRow
                key={folder.path}
                machine={machine}
                folder={folder}
                onOpen={() => {
                  setPath(folder.path);
                }}
              />
            ))}
            {data?.folders.length === 0 && (
              <p className="p-2 type-body">No folders here</p>
            )}
          </div>
        </ScrollArea>
      </div>
      <DialogFooter>
        <Button
          variant="ghost"
          size="sm"
          className="sm:mr-auto"
          disabled={!data || naming}
          onClick={() => {
            setNaming(true);
          }}
        >
          New folder
        </Button>
        <DialogClose render={<Button variant="ghost" size="sm" />}>
          Cancel
        </DialogClose>
        <Button
          variant="secondary"
          size="sm"
          disabled={!data || busy}
          aria-busy={busy}
          onClick={() => {
            if (data) {
              onChoose(data.path);
            }
          }}
        >
          {busy && <Spinner aria-hidden data-icon="inline-start" />}
          {action}
        </Button>
      </DialogFooter>
    </>
  );
}
