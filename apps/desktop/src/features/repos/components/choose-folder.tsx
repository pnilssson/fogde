import { FolderDialog } from "@/components/folder-dialog";
import { useForm } from "@tanstack/react-form";
import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { open as pick } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
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
import { reposFolderQuery } from "@/lib/folders";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  onBack: () => void;
  onContinue: () => Promise<void>;
}

export function ChooseFolder({ machine, onBack, onContinue }: Props) {
  const { data: saved } = useSuspenseQuery(reposFolderQuery(machine));
  const [browsing, setBrowsing] = useState(false);
  const queries = useQueryClient();
  const save = useMutation({
    mutationFn: (folder: string) =>
      machine.client.call("repos.setFolder", folder),
    onSuccess: async () => {
      await queries.query({ ...reposFolderQuery(machine), staleTime: 0 });
      await onContinue();
    },
    onError: (error) => {
      toast.add({
        type: "error",
        title: "Could not save the folder",
        description: error.message,
      });
    },
  });
  const form = useForm({
    defaultValues: { folder: saved },
    validators: {
      onSubmitAsync: async ({ value }) => {
        try {
          await machine.client.call("repos.checkFolder", value.folder);
          return null;
        } catch (error) {
          return {
            fields: {
              folder: {
                message: error instanceof Error ? error.message : String(error),
              },
            },
          };
        }
      },
    },
    onSubmit: ({ value }) => {
      save.mutate(value.folder);
    },
  });

  const browse = async () => {
    if (!machine.local) {
      setBrowsing(true);
      return;
    }

    const picked = await pick({
      directory: true,
      defaultPath: form.getFieldValue("folder") || undefined,
    });
    if (picked) {
      form.setFieldValue("folder", picked);
    }
  };

  return (
    <section className="flex max-w-sm flex-col items-start gap-4">
      <h1 className="type-display">Choose a repos folder</h1>
      <p className="type-body">
        Repositories are cloned here. Pick a folder you already use to keep your
        existing clones.
      </p>
      <form
        className="flex w-full flex-col gap-4"
        onSubmit={(event) => {
          event.preventDefault();
          event.stopPropagation();
          void form.handleSubmit();
        }}
      >
        <form.Field
          name="folder"
          validators={{
            onSubmit: z.string().trim().min(1, "Enter a folder"),
          }}
        >
          {(field) => (
            <Field data-invalid={!field.state.meta.isValid}>
              <FieldLabel htmlFor={field.name} className="sr-only">
                Repos folder
              </FieldLabel>
              <InputGroup>
                <InputGroupInput
                  id={field.name}
                  name={field.name}
                  value={field.state.value}
                  aria-invalid={!field.state.meta.isValid}
                  spellCheck={false}
                  onBlur={field.handleBlur}
                  onChange={(event) => {
                    field.handleChange(event.target.value);
                  }}
                />
                <InputGroupAddon align="inline-end">
                  <InputGroupButton onClick={() => void browse()}>
                    Browse
                  </InputGroupButton>
                </InputGroupAddon>
              </InputGroup>
              <FieldError errors={field.state.meta.errors} />
            </Field>
          )}
        </form.Field>
        <div className="flex gap-2">
          <Button type="button" variant="ghost" size="sm" onClick={onBack}>
            Back
          </Button>
          <form.Subscribe selector={(state) => state.isSubmitting}>
            {(submitting) => {
              const busy = submitting || save.isPending;

              return (
                <Button
                  type="submit"
                  variant="secondary"
                  size="sm"
                  aria-busy={busy}
                  disabled={busy}
                >
                  {busy && <Spinner aria-hidden data-icon="inline-start" />}
                  {busy ? "Saving…" : "Continue"}
                </Button>
              );
            }}
          </form.Subscribe>
        </div>
      </form>
      <FolderDialog
        machine={machine}
        open={browsing}
        start={form.getFieldValue("folder")}
        title="Choose a folder"
        action="Use this folder"
        onOpenChange={setBrowsing}
        onChoose={(path) => {
          form.setFieldValue("folder", path);
          setBrowsing(false);
        }}
      />
    </section>
  );
}
