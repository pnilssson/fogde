import { StepBars } from "@/components/step-bars";

interface Props {
  position: number;
  total: number;
}

export function OnboardingProgress({ position, total }: Props) {
  return (
    <div>
      <StepBars total={total} position={position} />
      <p className="sr-only">
        Step {position + 1} of {total}
      </p>
    </div>
  );
}
