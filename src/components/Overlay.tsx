import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useBuildOrder } from "../hooks/useBuildOrder";
import { useGameState } from "../hooks/useGameState";
import { StepCard } from "./StepCard";

interface OverlayProps {
  onOpenLibrary: () => void;
}

export function Overlay({ onOpenLibrary }: OverlayProps) {
  const { buildOrder, currentStep, totalSteps, advance, previous, reset } = useBuildOrder();
  const gameState = useGameState();
  const [capturing, setCapturing] = useState(false);

  const toggleCapture = async () => {
    try {
      if (capturing) {
        await invoke("stop_capture");
        setCapturing(false);
      } else {
        await invoke("start_capture");
        setCapturing(true);
      }
    } catch (e) {
      console.error("Capture toggle failed:", e);
    }
  };

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
      {gameState && (
        <div className="game-state-bar">
          {gameState.food != null && <span className="resource food">F:{gameState.food}</span>}
          {gameState.wood != null && <span className="resource wood">W:{gameState.wood}</span>}
          {gameState.gold != null && <span className="resource gold">G:{gameState.gold}</span>}
          {gameState.stone != null && <span className="resource stone">S:{gameState.stone}</span>}
          {gameState.villagers != null && <span className="resource vils">V:{gameState.villagers}</span>}
          {gameState.game_time_seconds != null && (
            <span className="resource time">
              {Math.floor(gameState.game_time_seconds / 60)}:{String(gameState.game_time_seconds % 60).padStart(2, "0")}
            </span>
          )}
        </div>
      )}
      <div className="overlay-nav">
        <button className="nav-btn" onClick={previous}>Prev</button>
        <button className="nav-btn" onClick={reset}>Reset</button>
        <button className="nav-btn" onClick={advance}>Next</button>
        <button className={`nav-btn ${capturing ? "nav-btn--active" : ""}`} onClick={toggleCapture}>
          {capturing ? "Stop" : "Capture"}
        </button>
      </div>
    </div>
  );
}
