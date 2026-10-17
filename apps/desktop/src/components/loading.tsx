import { Centered } from "@/components/centered";
import { Spinner } from "@/components/ui/spinner";

export function Loading() {
  return (
    <Centered>
      <Spinner />
    </Centered>
  );
}
