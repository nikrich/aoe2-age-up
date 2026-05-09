import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useGameState } from "../hooks/useGameState";
import type { Calibration } from "../lib/types";
import { formatGameTime } from "../lib/format";

const REGION_ORDER: Array<{ key: string; label: string }> = [
  { key: "Food", label: "Food" },
  { key: "Wood", label: "Wood" },
  { key: "Gold", label: "Gold" },
  { key: "Stone", label: "Stone" },
  { key: "Villagers", label: "Villagers" },
  { key: "Population", label: "Population" },
  { key: "GameTime", label: "Game time" },
];

export function CalibrationWizard() {
  const [cal, setCal] = useState<Calibration | null>(null);
  const [active, setActive] = useState<string>("Food");
  const [error, setError] = useState<string | null>(null);
  const [snapshotPath, setSnapshotPath] = useState<string | null>(null);
  const [snapshotBusy, setSnapshotBusy] = useState(false);
  const game = useGameState();

  useEffect(() => {
    invoke<Calibration>("get_calibration")
      .then(setCal)
      .catch((e) => setError(String(e)));
  }, []);

  const captureSnapshot = async () => {
    setSnapshotBusy(true);
    setError(null);
    try {
      const path = await invoke<string>("generate_calibration_image");
      setSnapshotPath(path);
    } catch (e) {
      setError(String(e));
    } finally {
      setSnapshotBusy(false);
    }
  };

  const valueFor = (key: string): string => {
    if (!game) return "—";
    switch (key) {
      case "Food": return game.food?.toString() ?? "—";
      case "Wood": return game.wood?.toString() ?? "—";
      case "Gold": return game.gold?.toString() ?? "—";
      case "Stone": return game.stone?.toString() ?? "—";
      case "Villagers": return game.villagers?.toString() ?? "—";
      case "Population":
        return game.population ? `${game.population[0]}/${game.population[1]}` : "—";
      case "GameTime":
        return formatGameTime(game.game_time_seconds);
      default: return "—";
    }
  };

  if (error && !cal) {
    return (
      <div className="calibration">
        <div className="lib-error">Calibration failed to load: {error}</div>
      </div>
    );
  }

  return (
    <div className="calibration">
      <div className="cal-header">
        <span className="cal-step">CALIBRATION</span>
        <div className="cal-title">Resource regions</div>
        <div className="cal-desc">
          {cal
            ? `Profile "${cal.profile_name}" · ${cal.resolution[0]}×${cal.resolution[1]} · ${REGION_ORDER.length} regions`
            : "Loading current calibration…"}
        </div>
      </div>

      <div className="cal-list">
        {REGION_ORDER.map(({ key, label }) => {
          const region = cal?.regions[key];
          const isActive = active === key;
          const isDone = !!region;
          return (
            <div
              key={key}
              className={`cal-item ${isActive ? "active" : ""} ${isDone ? "done" : ""}`}
              onClick={() => setActive(key)}
            >
              <div className="check">{isDone ? "✓" : ""}</div>
              <div className="nm">{label}</div>
              <div className="coords">
                {region
                  ? `${region.x},${region.y} ${region.width}×${region.height}`
                  : "—"}
              </div>
            </div>
          );
        })}
      </div>

      <div className="cal-readout">
        <div className="row"><span className="l">Live OCR</span><span className={`v ${game ? "ok" : "empty"}`}>{game ? "active" : "idle"}</span></div>
        {REGION_ORDER.map(({ key, label }) => {
          const v = valueFor(key);
          return (
            <div key={key} className="row">
              <span className="l">{label}</span>
              <span className={`v ${v === "—" ? "empty" : ""}`}>{v}</span>
            </div>
          );
        })}
      </div>

      <div className="cal-actions">
        <button
          className="btn primary flex"
          onClick={captureSnapshot}
          disabled={snapshotBusy}
        >
          {snapshotBusy ? "Capturing…" : "Capture sample image"}
        </button>
      </div>
      {snapshotPath && (
        <div className="cal-readout">
          <div className="row"><span className="l">Saved</span><span className="v">{snapshotPath}</span></div>
        </div>
      )}
      {error && cal && <div className="lib-error">{error}</div>}
    </div>
  );
}
