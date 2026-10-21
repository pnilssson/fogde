import { useForm } from "@tanstack/react-form";
import { useId } from "react";
import { z } from "zod";

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Field, FieldError, FieldLabel } from "@/components/ui/field";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@/components/ui/input-group";
import { Spinner } from "@/components/ui/spinner";
import { useCreateWorktree } from "@/features/workspaces/hooks/use-worktrees";
import { mainCheckout } from "@/features/workspaces/lib/checkouts";
import type { Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  workspace: Workspace;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function NewWorktree({ machine, workspace, open, onOpenChange }: Props) {
  const main = mainCheckout(workspace);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{`New worktree in ${workspace.name}`}</DialogTitle>
        </DialogHeader>
        {main && (
          <WorktreeForm
            machine={machine}
            workspace={workspace.id}
            repo={main.path}
            label="Name"
            onCreated={() => {
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
  workspace: string;
  repo: string;
  label: string;
  className?: string;
  onCreated?: () => void;
}

export function WorktreeForm({
  machine,
  workspace,
  repo,
  label,
  className,
  onCreated,
}: FormProps) {
  const id = useId();
  const create = useCreateWorktree(machine, workspace, onCreated);
  const form = useForm({
    defaultValues: { name: "" },
    onSubmit: ({ value }) =>
      create.mutateAsync({ repo, name: value.name.trim() }),
  });

  return (
    <form
      className={className}
      onSubmit={(event) => {
        event.preventDefault();
        event.stopPropagation();
        void form.handleSubmit().catch(() => undefined);
      }}
    >
      <form.Field
        name="name"
        validators={{ onSubmit: z.string().trim().min(1, "Enter a name") }}
      >
        {(field) => (
          <Field data-invalid={!field.state.meta.isValid}>
            <FieldLabel htmlFor={id}>{label}</FieldLabel>
            <InputGroup>
              <InputGroupInput
                id={id}
                name={field.name}
                value={field.state.value}
                placeholder="fix-login"
                spellCheck={false}
                autoCapitalize="off"
                aria-invalid={!field.state.meta.isValid}
                onBlur={field.handleBlur}
                onChange={(event) => {
                  field.handleChange(event.target.value);
                }}
              />
              <InputGroupAddon align="inline-end">
                <InputGroupButton
                  type="submit"
                  disabled={create.isPending}
                  aria-busy={create.isPending}
                >
                  {create.isPending && (
                    <Spinner aria-hidden data-icon="inline-start" />
                  )}
                  {create.isPending ? "Creating…" : "Create"}
                </InputGroupButton>
              </InputGroupAddon>
            </InputGroup>
            <FieldError errors={field.state.meta.errors} />
          </Field>
        )}
      </form.Field>
    </form>
  );
}
