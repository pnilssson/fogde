import { toast } from "@/components/ui/toast";

export function report(title: string, error: unknown) {
  toast.add({
    type: "error",
    title,
    description: error instanceof Error ? error.message : String(error),
  });
}
