import { useForm } from "@tanstack/react-form";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { z } from "zod";

import { Button } from "@/components/ui/button";
import { Field, FieldError, FieldLabel } from "@/components/ui/field";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@/components/ui/input-group";
import { Spinner } from "@/components/ui/spinner";
import { toast } from "@/components/ui/toast";
import { workspacesQuery } from "@/features/workspaces/api/workspaces";
import { checkoutOf } from "@/features/workspaces/lib/checkouts";
import { openStart } from "@/features/workspaces/lib/tiles";
import type { Checkout, Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  workspace: Workspace;
  folder: string;
}

export function WorktreeActions({ machine, workspace, folder }: Props) {
  const main = workspace.checkouts.find((checkout) => checkout.main);
  const checkout = checkoutOf(workspace, folder);
  if (!main || !checkout) {
    return null;
  }

  if (checkout.main) {
    return <NewWorktree machine={machine} repo={main.path} />;
  }

  return (
    <RemoveWorktree
      machine={machine}
      repo={main.path}
      checkout={checkout}
      open={workspace.terminals.some(
        (terminal) => terminal.checkout === checkout.path,
      )}
    />
  );
}

interface NewProps {
  machine: Machine;
  repo: string;
}

function NewWorktree({ machine, repo }: NewProps) {
  const queries = useQueryClient();
  const create = useMutation({
    mutationFn: (name: string) =>
      machine.client.call("worktrees.create", { repo, name }),
    onSuccess: async (path) => {
      await queries.invalidateQueries(workspacesQuery(machine));
      openStart(path);
    },
    onError: (error) => {
      toast.add({
        type: "error",
        title: "Could not create the worktree",
        description: error.message,
      });
    },
  });
  const form = useForm({
    defaultValues: { name: "" },
    onSubmit: ({ value }) => create.mutateAsync(value.name.trim()),
  });

  return (
    <form
      className="w-full max-w-xs"
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
            <FieldLabel htmlFor="worktree">New worktree</FieldLabel>
            <InputGroup>
              <InputGroupInput
                id="worktree"
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

interface RemoveProps {
  machine: Machine;
  repo: string;
  checkout: Checkout;
  open: boolean;
}

function RemoveWorktree({ machine, repo, checkout, open }: RemoveProps) {
  const queries = useQueryClient();
  const remove = useMutation({
    mutationFn: () =>
      machine.client.call("worktrees.remove", { repo, path: checkout.path }),
    onSuccess: async () => {
      await queries.invalidateQueries(workspacesQuery(machine));
      openStart(repo);
    },
    onError: (error) => {
      toast.add({
        type: "error",
        title: "Could not remove the worktree",
        description: error.message,
      });
    },
  });

  return (
    <div className="flex flex-col items-center gap-2">
      <Button
        variant="ghost"
        size="sm"
        disabled={open || remove.isPending}
        aria-busy={remove.isPending}
        onClick={() => {
          remove.mutate();
        }}
      >
        {remove.isPending && <Spinner aria-hidden data-icon="inline-start" />}
        {remove.isPending ? "Removing…" : "Remove worktree"}
      </Button>
      {open && (
        <p className="type-caption">Close its terminals to remove it.</p>
      )}
    </div>
  );
}
