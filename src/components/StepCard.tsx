import type { Step } from "../lib/types";
import { formatTrigger } from "../lib/format";

interface StepCardProps {
  step: Step;
  variant: "current" | "next";
}

export function StepCard({ step, variant }: StepCardProps) {
  const at = formatTrigger(step.at);
  const v = step.villagers_assigned;

  return (
    <div className={`step-card ${variant}`}>
      <div className="step-meta">
        <span className="step-tag">{variant === "current" ? "Now" : "Next"}</span>
        {at && <span className="at">{at}</span>}
      </div>
      <div className="step-action">{step.action}</div>
      {variant === "current" && step.notes && (
        <div className="step-notes">{step.notes}</div>
      )}
      {variant === "current" && v && (
        <div className="step-villagers">
          {v.food > 0 && (
            <span className="vchip food"><span className="lbl">F:</span><span className="num">{v.food}</span></span>
          )}
          {v.wood > 0 && (
            <span className="vchip wood"><span className="lbl">W:</span><span className="num">{v.wood}</span></span>
          )}
          {v.gold > 0 && (
            <span className="vchip gold"><span className="lbl">G:</span><span className="num">{v.gold}</span></span>
          )}
          {v.stone > 0 && (
            <span className="vchip stone"><span className="lbl">S:</span><span className="num">{v.stone}</span></span>
          )}
        </div>
      )}
    </div>
  );
}
