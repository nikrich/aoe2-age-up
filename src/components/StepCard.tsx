import type { Step } from "../lib/types";

interface StepCardProps {
  step: Step;
  variant: "current" | "next" | "default";
}

export function StepCard({ step, variant }: StepCardProps) {
  return (
    <div className={`step-card step-card--${variant}`}>
      <div className="step-action">{step.action}</div>
      {step.notes && <div className="step-notes">{step.notes}</div>}
      {step.villagers_assigned && (
        <div className="step-villagers">
          {step.villagers_assigned.food > 0 && <span className="step-villager-item">F:{step.villagers_assigned.food}</span>}
          {step.villagers_assigned.wood > 0 && <span className="step-villager-item">W:{step.villagers_assigned.wood}</span>}
          {step.villagers_assigned.gold > 0 && <span className="step-villager-item">G:{step.villagers_assigned.gold}</span>}
          {step.villagers_assigned.stone > 0 && <span className="step-villager-item">S:{step.villagers_assigned.stone}</span>}
        </div>
      )}
    </div>
  );
}
