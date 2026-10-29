import { useForm } from "@tanstack/react-form";
import { type ReactNode, useId } from "react";
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
import { Textarea } from "@/components/ui/textarea";

interface Text {
  title: string;
  body: string;
}

interface Props {
  open: boolean;
  heading: string;
  action: string;
  busy: string;
  initial?: Text;
  children?: ReactNode;
  onOpenChange: (open: boolean) => void;
  onSubmit: (text: Text) => Promise<unknown>;
}

export function TextDialog({ open, heading, onOpenChange, ...form }: Props) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>{heading}</DialogTitle>
        </DialogHeader>
        <TextForm {...form} />
      </DialogContent>
    </Dialog>
  );
}

type FormProps = Omit<Props, "open" | "heading" | "onOpenChange">;

function TextForm({
  action,
  busy,
  initial = { title: "", body: "" },
  children,
  onSubmit,
}: FormProps) {
  const id = useId();
  const form = useForm({
    defaultValues: initial,
    onSubmit: ({ value }) =>
      onSubmit({ title: value.title.trim(), body: value.body }),
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
      {children}
      <form.Field
        name="title"
        validators={{ onSubmit: z.string().trim().min(1, "Enter a title") }}
      >
        {(field) => (
          <Field data-invalid={!field.state.meta.isValid}>
            <FieldLabel htmlFor={`${id}-title`}>Title</FieldLabel>
            <Input
              id={`${id}-title`}
              name={field.name}
              value={field.state.value}
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
      <form.Field name="body">
        {(field) => (
          <Field>
            <FieldLabel htmlFor={`${id}-body`}>Description</FieldLabel>
            <Textarea
              id={`${id}-body`}
              name={field.name}
              value={field.state.value}
              className="max-h-80 min-h-32"
              onBlur={field.handleBlur}
              onChange={(event) => {
                field.handleChange(event.target.value);
              }}
            />
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
