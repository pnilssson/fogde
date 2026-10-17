import { StepBars } from "@/components/step-bars";

const steps = ["github", "agents"] as const;

interface Props {
  step: (typeof steps)[number];
}

export function OnboardingProgress({ step }: Props) {
  const position = steps.indexOf(step);

  return (
    <div>
      <StepBars total={steps.length} position={position} />
      <p className="sr-only">
        Step {position + 1} of {steps.length}
      </p>
    </div>
  );
}
