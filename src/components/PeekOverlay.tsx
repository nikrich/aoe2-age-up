import { useBuildOrder } from "../hooks/useBuildOrder";
import { useGameState } from "../hooks/useGameState";
import type { BuildOrder } from "../lib/types";

interface PeekProps {
  buildOrder: BuildOrder | null;
  onExpand: () => void;
}

export function PeekOverlay({ buildOrder, onExpand }: PeekProps) {
  const { currentStep } = useBuildOrder();
  const game = useGameState();

  if (!buildOrder) {
    return (
      <div className="peek" data-tauri-drag-region>
        <span className="ix">—</span>
        <span className="act">No build order loaded</span>
        <button className="expand" onClick={onExpand} title="Expand">▾</button>
      </div>
    );
  }

  const total = buildOrder.steps.length;
  const idx = Math.min(currentStep, total - 1);
  const step = buildOrder.steps[idx];

  const popNow = game?.population?.[0];
  const popMax = game?.population?.[1];

  return (
    <div className="peek" data-tauri-drag-region>
      <span className="ix">{idx + 1}/{total}</span>
      <span className="act">{step?.action ?? ""}</span>
      {popNow != null && popMax != null && (
        <span className="pop-mini">{popNow}/{popMax}</span>
      )}
      <button className="expand" onClick={onExpand} title="Expand">▾</button>
    </div>
  );
}
