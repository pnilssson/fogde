import { FolderIcon } from "@phosphor-icons/react";
import { useForm } from "@tanstack/react-form";
import { useQuery } from "@tanstack/react-query";
import { open as pick } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { z } from "zod";

import { FolderDialog } from "@/components/folder-dialog";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Field, FieldError, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Spinner } from "@/components/ui/spinner";
import { useCreateWorkspace } from "@/features/workspaces/hooks/use-workspaces";
import { folderName, foldersQuery, reposFolderQuery } from "@/lib/folders";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function NewWorkspace({ machine, open, onOpenChange }: Props) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>New workspace</DialogTitle>
        </DialogHeader>
        {open && (
          <WorkspaceForm
            machine={machine}
            onDone={() => {
              onOpenChange(false);
            }}
          />
        )}
      </DialogContent>
    </Dialog>
  );
}

interface FormProps {
  machine: Machine;
  onDone: () => void;
}

function WorkspaceForm({ machine, onDone }: FormProps) {
  const create = useCreateWorkspace(machine, onDone);
  const { data: repos } = useQuery(reposFolderQuery(machine));
  const listing = useQuery({
    ...foldersQuery(machine, repos ?? ""),
    enabled: repos !== undefined,
  });
  const [browsing, setBrowsing] = useState(false);
  const form = useForm({
    defaultValues: { folder: "", name: "" },
    onSubmit: ({ value }) =>
      create.mutateAsync({ name: value.name.trim(), folder: value.folder }),
  });

  const choose = (path: string) => {
    const name = form.getFieldValue("name");
    if (!name || name === folderName(form.getFieldValue("folder"))) {
      form.setFieldValue("name", folderName(path));
    }
    form.setFieldValue("folder", path);
    create.reset();
  };

  const browse = async () => {
    if (!machine.local) {
      setBrowsing(true);
      return;
    }

    const picked = await pick({ directory: true, defaultPath: repos });
    if (picked) {
      choose(picked);
    }
  };

  return (
    <>
      <form
        className="flex flex-col gap-4"
        onSubmit={(event) => {
          event.preventDefault();
          event.stopPropagation();
          void form.handleSubmit().catch(() => undefined);
        }}
      >
        <form.Field
          name="folder"
          validators={{ onSubmit: z.string().min(1, "Choose a folder") }}
        >
          {(field) => {
            const listed = listing.data?.folders.some(
              ({ path }) => path === field.state.value,
            );

            return (
              <Field data-invalid={!field.state.meta.isValid}>
                <FieldLabel>Folder</FieldLabel>
                <ScrollArea className="h-48">
                  <div className="flex flex-col">
                    {listing.isPending && <Spinner aria-hidden />}
                    {listing.data?.folders.map((folder) => (
                      <Button
                        key={folder.path}
                        type="button"
                        variant={
                          field.state.value === folder.path
                            ? "secondary"
                            : "ghost"
                        }
                        size="sm"
                        className="justify-start"
                        aria-pressed={field.state.value === folder.path}
                        onClick={() => {
                          choose(folder.path);
                        }}
                      >
                        <FolderIcon data-icon="inline-start" />
                        <span className="truncate">{folder.name}</span>
                      </Button>
                    ))}
                  </div>
                </ScrollArea>
                {field.state.value && !listed && (
                  <p className="truncate type-caption text-muted-foreground">
                    {field.state.value}
                  </p>
                )}
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="self-start"
                  onClick={() => void browse()}
                >
                  Other folder…
                </Button>
                <FieldError errors={field.state.meta.errors} />
              </Field>
            );
          }}
        </form.Field>
        <form.Field
          name="name"
          validators={{ onSubmit: z.string().trim().min(1, "Enter a name") }}
        >
          {(field) => (
            <Field data-invalid={!field.state.meta.isValid || create.isError}>
              <FieldLabel htmlFor={field.name}>Name</FieldLabel>
              <Input
                id={field.name}
                name={field.name}
                value={field.state.value}
                placeholder="fogde"
                aria-invalid={!field.state.meta.isValid || create.isError}
                onBlur={field.handleBlur}
                onChange={(event) => {
                  field.handleChange(event.target.value);
                }}
              />
              <FieldError
                errors={[...field.state.meta.errors, create.error ?? undefined]}
              />
            </Field>
          )}
        </form.Field>
        <DialogFooter>
          <Button type="submit" size="sm" disabled={create.isPending}>
            {create.isPending && (
              <Spinner aria-hidden data-icon="inline-start" />
            )}
            {create.isPending ? "Creating…" : "Create"}
          </Button>
        </DialogFooter>
      </form>
      <FolderDialog
        machine={machine}
        open={browsing}
        start={repos ?? ""}
        onOpenChange={setBrowsing}
        onChoose={(path) => {
          choose(path);
          setBrowsing(false);
        }}
      />
    </>
  );
}
