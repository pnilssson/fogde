import { BarTitle } from "@/components/bar-title";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  machines: Machine[];
  onMachine: (machine: string) => void;
  named?: boolean;
}

export function MachinePicker({
  machine,
  machines,
  onMachine,
  named = false,
}: Props) {
  if (machines.length < 2) {
    return named ? <BarTitle title={machine.name} muted /> : null;
  }

  return (
    <Select
      items={machines.map(({ id, name }) => ({ label: name, value: id }))}
      value={machine.id}
      onValueChange={(next) => {
        if (next) {
          onMachine(next);
        }
      }}
    >
      <SelectTrigger size="sm" aria-label="Machine" className="w-fit min-w-0">
        <SelectValue className="block truncate" />
      </SelectTrigger>
      <SelectContent>
        {machines.map(({ id, name }) => (
          <SelectItem key={id} value={id}>
            {name}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
