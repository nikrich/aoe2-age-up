import { useState, useEffect, useRef, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useGameState } from "../hooks/useGameState";
import type { Calibration, Region } from "../lib/types";
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

// Visible viewport (the scroll container). Sized to fill the wide Cal window
// (1200 wide; ~860 canvas + ~310 sidebar + padding/gap).
const CANVAS_VIEWPORT_WIDTH = 860;
const CANVAS_VIEWPORT_HEIGHT = 600;
// How many CSS pixels per native pixel. Higher = bigger regions, more scroll.
const ZOOM = 2.0;
const MIN_REGION_PX = 8; // native px

type Handle = "tl" | "tr" | "bl" | "br" | "body";

interface DragState {
  regionKey: string;
  handle: Handle;
  startPointerX: number;
  startPointerY: number;
  startRegion: Region;
}

function clamp(v: number, lo: number, hi: number) {
  return Math.max(lo, Math.min(hi, v));
}

function coordsLabel(r: Region) {
  return `${r.x},${r.y} ${r.width}×${r.height}`;
}

export function CalibrationWizard() {
  const [cal, setCal] = useState<Calibration | null>(null);
  const [regions, setRegions] = useState<Record<string, Region>>({});
  const [active, setActive] = useState<string>("Food");
  const [error, setError] = useState<string | null>(null);
  const [info, setInfo] = useState<string | null>(null);
  const [snapshot, setSnapshot] = useState<string | null>(null);
  const [snapshotBusy, setSnapshotBusy] = useState(false);
  const [saveBusy, setSaveBusy] = useState(false);
  const [dirty, setDirty] = useState(false);

  const dragRef = useRef<DragState | null>(null);
  const game = useGameState();

  // Load current calibration once.
  useEffect(() => {
    invoke<Calibration>("get_calibration")
      .then((c) => {
        setCal(c);
        setRegions(c.regions);
      })
      .catch((e) => setError(String(e)));
  }, []);

  const nativeW = cal?.resolution[0] ?? 1920;
  const nativeH = cal?.resolution[1] ?? 1080;
  // Scale = CSS pixels per native pixel. Independent of viewport size — the
  // viewport just clips and scrolls. Higher zoom keeps regions grabbable.
  const scale = useMemo(() => (ZOOM * CANVAS_VIEWPORT_WIDTH) / nativeW, [nativeW]);
  const displayW = Math.round(nativeW * scale);
  const displayH = Math.round(nativeH * scale);

  const captureSnapshot = useCallback(async () => {
    setSnapshotBusy(true);
    setError(null);
    setInfo(null);
    try {
      const dataUri = await invoke<string>("capture_calibration_screenshot");
      setSnapshot(dataUri);
    } catch (e) {
      setError(String(e));
    } finally {
      setSnapshotBusy(false);
    }
  }, []);

  const save = useCallback(async () => {
    if (!cal) return;
    setSaveBusy(true);
    setError(null);
    try {
      const next: Calibration = { ...cal, regions };
      await invoke("set_calibration", { calibration: next });
      setCal(next);
      setDirty(false);
      setInfo("Calibration saved.");
    } catch (e) {
      setError(String(e));
    } finally {
      setSaveBusy(false);
    }
  }, [cal, regions]);

  const reset = useCallback(() => {
    if (!cal) return;
    setRegions(cal.regions);
    setDirty(false);
    setInfo("Reverted unsaved changes.");
  }, [cal]);

  const onPointerDown = (
    e: React.PointerEvent<HTMLDivElement>,
    regionKey: string,
    handle: Handle,
  ) => {
    e.stopPropagation();
    e.preventDefault();
    const region = regions[regionKey];
    if (!region) return;
    setActive(regionKey);
    dragRef.current = {
      regionKey,
      handle,
      startPointerX: e.clientX,
      startPointerY: e.clientY,
      startRegion: { ...region },
    };
    (e.target as HTMLDivElement).setPointerCapture?.(e.pointerId);
  };

  const onPointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    const d = dragRef.current;
    if (!d) return;
    const dxNative = (e.clientX - d.startPointerX) / scale;
    const dyNative = (e.clientY - d.startPointerY) / scale;

    setRegions((prev) => {
      const start = d.startRegion;
      let { x, y, width, height } = start;
      switch (d.handle) {
        case "body":
          x = start.x + dxNative;
          y = start.y + dyNative;
          break;
        case "tl":
          x = start.x + dxNative;
          y = start.y + dyNative;
          width = start.width - dxNative;
          height = start.height - dyNative;
          break;
        case "tr":
          y = start.y + dyNative;
          width = start.width + dxNative;
          height = start.height - dyNative;
          break;
        case "bl":
          x = start.x + dxNative;
          width = start.width - dxNative;
          height = start.height + dyNative;
          break;
        case "br":
          width = start.width + dxNative;
          height = start.height + dyNative;
          break;
      }
      // Enforce min size and keep within screen bounds.
      width = Math.max(MIN_REGION_PX, width);
      height = Math.max(MIN_REGION_PX, height);
      x = clamp(x, 0, Math.max(0, nativeW - width));
      y = clamp(y, 0, Math.max(0, nativeH - height));
      width = Math.min(width, nativeW - x);
      height = Math.min(height, nativeH - y);

      return {
        ...prev,
        [d.regionKey]: {
          x: Math.round(x),
          y: Math.round(y),
          width: Math.round(width),
          height: Math.round(height),
        },
      };
    });
    setDirty(true);
  };

  const onPointerUp = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!dragRef.current) return;
    (e.target as HTMLDivElement).releasePointerCapture?.(e.pointerId);
    dragRef.current = null;
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
      <div className="cal-grid">
        <div
          className="cal-viewport"
          style={{ width: CANVAS_VIEWPORT_WIDTH, height: CANVAS_VIEWPORT_HEIGHT }}
        >
        <div
          className="cal-canvas"
          style={{ width: displayW, height: displayH }}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerCancel={onPointerUp}
        >
          {snapshot ? (
            <img
              src={snapshot}
              alt="screenshot"
              className="cal-shot"
              style={{ width: displayW, height: displayH }}
            />
          ) : (
            <div className="cal-shot-placeholder">
              No screenshot. Click <b>Capture</b> to grab one.
            </div>
          )}
          {REGION_ORDER.map(({ key, label }) => {
            const r = regions[key];
            if (!r) return null;
            const isActive = active === key;
            return (
              <div
                key={key}
                className={`cal-region ${isActive ? "active" : ""}`}
                style={{
                  left: r.x * scale,
                  top: r.y * scale,
                  width: r.width * scale,
                  height: r.height * scale,
                  zIndex: isActive ? 2 : 1,
                }}
                onPointerDown={(e) => onPointerDown(e, key, "body")}
              >
                <span className="lbl">{label}</span>
                <div className="handle tl" onPointerDown={(e) => onPointerDown(e, key, "tl")} />
                <div className="handle tr" onPointerDown={(e) => onPointerDown(e, key, "tr")} />
                <div className="handle bl" onPointerDown={(e) => onPointerDown(e, key, "bl")} />
                <div className="handle br" onPointerDown={(e) => onPointerDown(e, key, "br")} />
              </div>
            );
          })}
        </div>
        </div>

        <div className="cal-side">
          <div className="cal-header">
            <span className="cal-step">CALIBRATION · {nativeW}×{nativeH}</span>
            <div className="cal-title">Resource regions</div>
            <div className="cal-desc">
              Capture a screenshot, then drag each box over its in-game number. Resize with the corner handles.
            </div>
          </div>

          <div className="cal-list">
            {REGION_ORDER.map(({ key, label }) => {
              const r = regions[key];
              const isActive = active === key;
              return (
                <div
                  key={key}
                  className={`cal-item ${isActive ? "active" : ""} ${r ? "done" : ""}`}
                  onClick={() => setActive(key)}
                >
                  <div className="check">{r ? "✓" : ""}</div>
                  <div className="nm">{label}</div>
                  <div className="coords">{r ? coordsLabel(r) : "—"}</div>
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
            <button className="btn flex" onClick={captureSnapshot} disabled={snapshotBusy}>
              {snapshotBusy ? "Capturing…" : "Capture"}
            </button>
            <button className="btn flex" onClick={reset} disabled={!dirty || saveBusy}>
              Reset
            </button>
            <button className="btn primary flex" onClick={save} disabled={!dirty || saveBusy}>
              {saveBusy ? "Saving…" : "Save"}
            </button>
          </div>
          {info && <div className="cal-info">{info}</div>}
          {error && cal && <div className="lib-error">{error}</div>}
        </div>
      </div>
    </div>
  );
}
