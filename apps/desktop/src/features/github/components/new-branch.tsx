import { useForm } from "@tanstack/react-form";
import { useId } from "react";
import { z } from "zod";

import {
  Dialog,
  DialogContent,
  DialogDescription,
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
import { useWrite } from "@/features/github/hooks/use-write";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  path: string;
  from: string | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function NewBranch({ machine, path, from, open, onOpenChange }: Props) {
  const id = useId();
  const create = useWrite(machine, "Could not create the branch");
  const form = useForm({
    defaultValues: { name: "" },
    onSubmit: async ({ value, formApi }) => {
      await create.mutateAsync((client) =>
        client.call("checkouts.createBranch", {
          path,
          branch: value.name.trim(),
        }),
      );
      formApi.reset();
      onOpenChange(false);
    },
  });

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>New branch</DialogTitle>
          <DialogDescription>
            {from
              ? `Starts from ${from} and takes your uncommitted changes along.`
              : "Starts here and takes your uncommitted changes along."}
          </DialogDescription>
        </DialogHeader>
        <form
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
                <FieldLabel htmlFor={id}>Name</FieldLabel>
                <InputGroup>
                  <InputGroupInput
                    id={id}
                    name={field.name}
                    value={field.state.value}
                    placeholder="feature/login"
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
      </DialogContent>
    </Dialog>
  );
}
