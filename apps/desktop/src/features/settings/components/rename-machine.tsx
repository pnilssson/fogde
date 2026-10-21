import { useForm } from "@tanstack/react-form";

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
import { useRenameMachine } from "@/features/settings/hooks/use-rename-machine";
import { computer, type Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function RenameMachine({ machine, open, onOpenChange }: Props) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{`Rename ${machine.name}`}</DialogTitle>
        </DialogHeader>
        {open && (
          <NameForm
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

function NameForm({ machine, onDone }: FormProps) {
  const rename = useRenameMachine(machine, onDone);
  const form = useForm({
    defaultValues: { name: machine.name },
    onSubmit: ({ value }) => rename.mutateAsync(value.name),
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
      <form.Field name="name">
        {(field) => (
          <Field data-invalid={rename.isError}>
            <FieldLabel htmlFor={field.name}>Name</FieldLabel>
            <Input
              id={field.name}
              name={field.name}
              value={field.state.value}
              placeholder={machine.target ?? computer.name}
              aria-invalid={rename.isError}
              onBlur={field.handleBlur}
              onChange={(event) => {
                field.handleChange(event.target.value);
              }}
            />
            <FieldError errors={[rename.error ?? undefined]} />
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
