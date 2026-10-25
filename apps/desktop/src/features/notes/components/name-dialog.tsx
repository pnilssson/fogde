import { useForm } from "@tanstack/react-form";
import { useId } from "react";
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

interface Props {
  open: boolean;
  title: string;
  action: string;
  busy: string;
  placeholder: string;
  initial?: string;
  onOpenChange: (open: boolean) => void;
  onSubmit: (name: string) => Promise<unknown>;
}

export function NameDialog({ open, title, onOpenChange, ...form }: Props) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
        </DialogHeader>
        <NameForm {...form} />
      </DialogContent>
    </Dialog>
  );
}

type FormProps = Omit<Props, "open" | "title" | "onOpenChange">;

function NameForm({
  action,
  busy,
  placeholder,
  initial = "",
  onSubmit,
}: FormProps) {
  const id = useId();
  const form = useForm({
    defaultValues: { name: initial },
    onSubmit: ({ value }) => onSubmit(value.name.trim()),
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
      <form.Field
        name="name"
        validators={{ onSubmit: z.string().trim().min(1, "Enter a name") }}
      >
        {(field) => (
          <Field data-invalid={!field.state.meta.isValid}>
            <FieldLabel htmlFor={id}>Name</FieldLabel>
            <Input
              id={id}
              name={field.name}
              value={field.state.value}
              placeholder={placeholder}
              spellCheck={false}
              aria-invalid={!field.state.meta.isValid}
              onFocus={(event) => {
                event.currentTarget.select();
              }}
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
        <form.Subscribe selector={(state) => state.isSubmitting}>
          {(submitting) => (
            <Button
              type="submit"
              size="sm"
              disabled={submitting}
              aria-busy={submitting}
            >
              {submitting && <Spinner aria-hidden data-icon="inline-start" />}
              {submitting ? busy : action}
            </Button>
          )}
        </form.Subscribe>
      </DialogFooter>
    </form>
  );
}
