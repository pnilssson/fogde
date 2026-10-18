import { StepBars } from "@/components/step-bars";
import { steps } from "@/lib/onboarding";

interface Props {
  position: number;
}

export function OnboardingProgress({ position }: Props) {
  return (
    <div>
      <StepBars total={steps.length} position={position} />
      <p className="sr-only">
        Step {position + 1} of {steps.length}
      </p>
    </div>
  );
}
