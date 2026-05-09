import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useBuildOrder } from "../hooks/useBuildOrder";
import { useGameState } from "../hooks/useGameState";
import { StepCard } from "./StepCard";
import type { BuildOrder, Step } from "../lib/types";
import { formatDuration, formatGameTime, computeDelta } from "../lib/format";

interface OverlayProps {
  onOpenLibrary: () => void;
  buildOrder: BuildOrder | null;
  hotkeys?: {
    advance_step?: string;
    previous_step?: string;
    reset?: string;
    pause_capture?: string;
  };
}

const HOTKEY_HINTS = {
  prev: "Ctrl+Alt+←",
  reset: "Ctrl+Alt+R",
  next: "Ctrl+Alt+→",
  pause: "Ctrl+Alt+P",
};

function tickClass(step: Step, idx: number, current: number): string {
  const cls: string[] = ["tick"];
  if (idx < current) cls.push("done");
  else if (idx === current) cls.push("cur");
  if (step.phase) cls.push(step.phase);
  return cls.join(" ");
}

export function Overlay({ onOpenLibrary, buildOrder }: OverlayProps) {
  const { currentStep, advance, previous, reset } = useBuildOrder();
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
          <button className="open-lib-btn" onClick={onOpenLibrary}>Open Library</button>
        </div>
      </div>
    );
  }

  const total = buildOrder.steps.length;
  const safeIndex = Math.min(currentStep, total - 1);
  const cur = buildOrder.steps[safeIndex];
  const nxt = safeIndex < total - 1 ? buildOrder.steps[safeIndex + 1] : null;

  const rawElapsed = gameState?.game_time_seconds ?? null;
  // Reject implausible OCR readings so they don't poison the UI
  const elapsed =
    rawElapsed != null && rawElapsed >= 0 && rawElapsed <= 3 * 60 * 60
      ? rawElapsed
      : null;
  const target = cur?.target_time_seconds ?? null;
  const delta = computeDelta(elapsed, target);

  const popNow = gameState?.population?.[0] ?? null;
  const popMax = gameState?.population?.[1] ?? null;

  return (
    <div className="overlay">
      <div className="bo-header">
        <span className="name">{buildOrder.name}</span>
        {buildOrder.civilization && (
          <span className="civ">{buildOrder.civilization}</span>
        )}
        <span className="counter"><b>{safeIndex + 1}</b> / {total}</span>
      </div>

      <div className="timeline">
        {buildOrder.steps.map((s, i) => (
          <span
            key={i}
            className={tickClass(s, i, safeIndex)}
            title={s.action}
          />
        ))}
      </div>

      {(elapsed != null || target != null) && (
        <div className="time-bar">
          <span className="lbl">T</span>
          <span className="elapsed">{elapsed != null ? formatDuration(elapsed) : "—"}</span>
          {target != null && (
            <>
              <span className="lbl">/</span>
              <span className="target">{formatDuration(target)}</span>
            </>
          )}
          {delta && (
            <span className={`delta ${delta.behind ? "behind" : "ahead"}`}>
              {delta.sign}{delta.text}
            </span>
          )}
          {popNow != null && popMax != null && (
            <span className="pop">
              <span className="lbl">POP</span>
              <span className="num">{popNow}</span>
              <span className="max">/{popMax}</span>
            </span>
          )}
        </div>
      )}

      {cur && <StepCard step={cur} variant="current" />}
      {nxt && <StepCard step={nxt} variant="next" />}

      {gameState && (
        <div className="game-state">
          <div className="cell food"><span className="lbl">F</span><span className={`v ${gameState.food == null ? "empty" : ""}`}>{gameState.food ?? "—"}</span></div>
          <div className="cell wood"><span className="lbl">W</span><span className={`v ${gameState.wood == null ? "empty" : ""}`}>{gameState.wood ?? "—"}</span></div>
          <div className="cell gold"><span className="lbl">G</span><span className={`v ${gameState.gold == null ? "empty" : ""}`}>{gameState.gold ?? "—"}</span></div>
          <div className="cell stone"><span className="lbl">S</span><span className={`v ${gameState.stone == null ? "empty" : ""}`}>{gameState.stone ?? "—"}</span></div>
          <div className="cell vils"><span className="lbl">V</span><span className={`v ${gameState.villagers == null ? "empty" : ""}`}>{gameState.villagers ?? "—"}</span></div>
          <div className="cell time"><span className="lbl">T</span><span className={`v ${elapsed == null ? "empty" : ""}`}>{formatGameTime(elapsed)}</span></div>
        </div>
      )}

      <div className="overlay-nav">
        <button className="nav-btn" onClick={previous}>
          <span>Prev</span>
          <span className="key">{HOTKEY_HINTS.prev}</span>
        </button>
        <button className="nav-btn" onClick={reset}>
          <span>Reset</span>
          <span className="key">{HOTKEY_HINTS.reset}</span>
        </button>
        <button className="nav-btn" onClick={advance}>
          <span>Next</span>
          <span className="key">{HOTKEY_HINTS.next}</span>
        </button>
        <button className={`nav-btn ${capturing ? "on" : ""}`} onClick={toggleCapture}>
          <span>{capturing ? "● Live" : "Capture"}</span>
          <span className="key">{HOTKEY_HINTS.pause}</span>
        </button>
      </div>
    </div>
  );
}
