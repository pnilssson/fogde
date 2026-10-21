import { Switch } from "@/components/ui/switch";
import { chooseMaterial, useMaterial } from "@/lib/theme";

export function TranslucentSwitch() {
  const material = useMaterial();

  return (
    <Switch
      checked={material === "translucent"}
      onCheckedChange={(on) => {
        void chooseMaterial(on ? "translucent" : "solid");
      }}
    />
  );
}
