import { useForm } from "@tanstack/react-form";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { z } from "zod";

import { Field, FieldError } from "@/components/ui/field";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@/components/ui/input-group";
import { Spinner } from "@/components/ui/spinner";
import { toast } from "@/components/ui/toast";
import type { Machine } from "@/lib/machines";

const Name = z.string().trim().min(1, "Enter a name for the folder");

interface Props {
  machine: Machine;
  parent: string;
  onCreated: (path: string) => void;
}

export function NewFolder({ machine, parent, onCreated }: Props) {
  const queries = useQueryClient();
  const make = useMutation({
    mutationFn: (name: string) =>
      machine.client.call("folders.create", { parent, name }),
    onSuccess: (folder) => {
      void queries.invalidateQueries({
        queryKey: [machine.id, "folders.list"],
      });
      onCreated(folder.path);
    },
    onError: (error) => {
      toast.add({
        type: "error",
        title: "Could not create the folder",
        description: error.message,
      });
    },
  });
  const form = useForm({
    defaultValues: { name: "" },
    onSubmit: ({ value }) => make.mutateAsync(value.name.trim()),
  });

  return (
    <form
      onSubmit={(event) => {
        event.preventDefault();
        event.stopPropagation();
        void form.handleSubmit().catch(() => undefined);
      }}
    >
      <form.Field name="name" validators={{ onSubmit: Name }}>
        {(field) => (
          <Field data-invalid={!field.state.meta.isValid}>
            <InputGroup>
              <InputGroupInput
                name={field.name}
                value={field.state.value}
                aria-label="New folder name"
                aria-invalid={!field.state.meta.isValid}
                placeholder="New folder"
                spellCheck={false}
                autoFocus
                onBlur={field.handleBlur}
                onChange={(event) => {
                  field.handleChange(event.target.value);
                }}
              />
              <InputGroupAddon align="inline-end">
                <InputGroupButton
                  type="submit"
                  size="xs"
                  disabled={make.isPending}
                  aria-busy={make.isPending}
                >
                  {make.isPending && <Spinner aria-hidden />}
                  Create
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
