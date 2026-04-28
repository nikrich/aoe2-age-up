import { useBuildOrder } from "../hooks/useBuildOrder";
import { StepCard } from "./StepCard";

interface OverlayProps {
  onOpenLibrary: () => void;
}

export function Overlay({ onOpenLibrary }: OverlayProps) {
  const { buildOrder, currentStep, totalSteps, advance, previous, reset } = useBuildOrder();

  if (!buildOrder) {
    return (
      <div className="overlay">
        <div className="no-build-order">
          <div>No build order loaded</div>
          <button className="nav-btn" onClick={onOpenLibrary}>Open Library</button>
        </div>
      </div>
    );
  }

  const currentStepData = buildOrder.steps[currentStep];
  const nextStepData = currentStep < buildOrder.steps.length - 1 ? buildOrder.steps[currentStep + 1] : null;

  return (
    <div className="overlay">
      <div className="overlay-header">
        <span className="overlay-title">{buildOrder.name}</span>
        <span className="step-counter">{currentStep + 1} / {totalSteps}</span>
      </div>
      {currentStepData && <StepCard step={currentStepData} variant="current" />}
      {nextStepData && <StepCard step={nextStepData} variant="next" />}
      <div className="overlay-nav">
        <button className="nav-btn" onClick={previous}>Prev</button>
        <button className="nav-btn" onClick={reset}>Reset</button>
        <button className="nav-btn" onClick={advance}>Next</button>
      </div>
    </div>
  );
}
