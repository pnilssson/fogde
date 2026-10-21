import { useForm } from "@tanstack/react-form";
import { z } from "zod";

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
import { Spinner } from "@/components/ui/spinner";
import { useRenameWorkspace } from "@/features/workspaces/hooks/use-workspaces";
import type { Workspace } from "@/generated/protocol";
import { folderName } from "@/lib/folders";
import type { Machine } from "@/lib/machines";

const Name = z.string().trim().min(1, "Enter a name for the workspace");

interface Props {
  machine: Machine;
  workspace: Workspace;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function RenameWorkspace({
  machine,
  workspace,
  open,
  onOpenChange,
}: Props) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{`Rename ${workspace.name}`}</DialogTitle>
        </DialogHeader>
        {open && (
          <NameForm
            machine={machine}
            workspace={workspace}
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
  workspace: Workspace;
  onDone: () => void;
}

function NameForm({ machine, workspace, onDone }: FormProps) {
  const rename = useRenameWorkspace(machine, onDone);
  const form = useForm({
    defaultValues: { name: workspace.name },
    onSubmit: ({ value }) =>
      rename.mutateAsync({ workspace: workspace.id, name: value.name.trim() }),
  });

  return (
    <form
      className="flex flex-col gap-4"
      onSubmit={(event) => {
        event.preventDefault();
        event.stopPropagation();
        void form.handleSubmit().catch(() => undefined);
      }}
    >
      <form.Field name="name" validators={{ onSubmit: Name }}>
        {(field) => (
          <Field data-invalid={!field.state.meta.isValid}>
            <FieldLabel htmlFor={field.name}>Name</FieldLabel>
            <Input
              id={field.name}
              name={field.name}
              value={field.state.value}
              placeholder={folderName(workspace.folder)}
              aria-invalid={!field.state.meta.isValid}
              onBlur={field.handleBlur}
              onChange={(event) => {
                field.handleChange(event.target.value);
              }}
            />
            <FieldError errors={field.state.meta.errors} />
          </Field>
        )}
      </form.Field>
      <DialogFooter>
        <Button type="submit" size="sm" disabled={rename.isPending}>
          {rename.isPending && <Spinner aria-hidden data-icon="inline-start" />}
          {rename.isPending ? "Renaming…" : "Rename"}
        </Button>
      </DialogFooter>
    </form>
  );
}
