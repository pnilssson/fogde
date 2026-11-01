import { useForm } from "@tanstack/react-form";
import { useId } from "react";
import { z } from "zod";

import { ActionButton } from "@/components/action-button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Field, FieldError, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";

interface Props {
  open: boolean;
  title: string;
  description?: string;
  action: string;
  busy: string;
  placeholder: string;
  initial?: string;
  optional?: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (name: string) => Promise<unknown>;
}

export function NameDialog({
  open,
  title,
  description,
  onOpenChange,
  ...form
}: Props) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          {description && <DialogDescription>{description}</DialogDescription>}
        </DialogHeader>
        <NameForm {...form} />
      </DialogContent>
    </Dialog>
  );
}

type FormProps = Omit<Props, "open" | "title" | "description" | "onOpenChange">;

const named = z.string().trim().min(1, "Enter a name");

function NameForm({
  action,
  busy,
  placeholder,
  initial = "",
  optional = false,
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
      <form.Field name="name" validators={optional ? {} : { onSubmit: named }}>
        {(field) => (
          <Field data-invalid={!field.state.meta.isValid}>
            <FieldLabel htmlFor={id}>Name</FieldLabel>
            <Input
              id={id}
              name={field.name}
              value={field.state.value}
              placeholder={placeholder}
              spellCheck={false}
              autoCapitalize="off"
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
            <ActionButton
              type="submit"
              variant="default"
              label={action}
              busy={busy}
              pending={submitting}
            />
          )}
        </form.Subscribe>
      </DialogFooter>
    </form>
  );
}
