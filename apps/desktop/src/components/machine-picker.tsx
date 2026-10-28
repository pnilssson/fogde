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
}

export function MachinePicker({ machine, machines, onMachine }: Props) {
  if (machines.length < 2) {
    return null;
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
      <SelectTrigger size="sm" aria-label="Machine" className="w-fit">
        <SelectValue />
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
