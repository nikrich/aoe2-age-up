import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { currentMonitor, getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";
import { Overlay } from "./components/Overlay";
import { PeekOverlay } from "./components/PeekOverlay";
import { BuildOrderLibrary } from "./components/BuildOrderLibrary";
import { Settings } from "./components/Settings";
import { CalibrationWizard } from "./components/CalibrationWizard";
import type { BuildOrder } from "./lib/types";
import "./styles/overlay.css";

type View = "run" | "library" | "calibration" | "settings";

const MIN_W = 200;
const MIN_H = 40;

// Calibration view layout — fixed window size, centered horizontally on the
// active monitor with a top margin so the game's resource bar stays visible.
const CAL_WIN_W = 1200;
const CAL_WIN_H = 720;
const CAL_TOP_MARGIN = 120;

// localStorage key for the user's preferred overlay position (logical px).
// Only saved when *not* in Calibration mode so Cal's snap-to-center doesn't
// poison the persisted position.
const POSITION_STORAGE_KEY = "open-age:overlay-pos";
const POSITION_SAVE_DEBOUNCE_MS = 400;

function App() {
  const [view, setView] = useState<View>("run");
  const [buildOrder, setBuildOrder] = useState<BuildOrder | null>(null);
  const [peek, setPeek] = useState(false);

  // Suppress the webview's right-click context menu (Refresh / Inspect / etc.)
  useEffect(() => {
    const handler = (e: MouseEvent) => e.preventDefault();
    window.addEventListener("contextmenu", handler);
    return () => window.removeEventListener("contextmenu", handler);
  }, []);

  // Restore the user's last overlay position on startup.
  useEffect(() => {
    const saved = localStorage.getItem(POSITION_STORAGE_KEY);
    if (!saved) return;
    try {
      const { x, y } = JSON.parse(saved);
      if (typeof x !== "number" || typeof y !== "number") return;
      getCurrentWindow()
        .setPosition(new LogicalPosition(x, y))
        .catch((e) => console.error("Restore startup position failed:", e));
    } catch (e) {
      console.error("Bad saved position:", e);
    }
  }, []);

  // Persist the overlay position whenever the user moves the window — but skip
  // moves that happen while in Cal (it snap-centers and shouldn't overwrite
  // the user's preferred overlay spot).
  const viewRef = useRef<View>(view);
  viewRef.current = view;
  useEffect(() => {
    const win = getCurrentWindow();
    let timer: number | null = null;
    const unlistenP = win.onMoved(async ({ payload }) => {
      if (viewRef.current === "calibration") return;
      if (timer) window.clearTimeout(timer);
      timer = window.setTimeout(async () => {
        try {
          const monitor = await currentMonitor();
          const scale = monitor?.scaleFactor ?? 1;
          const x = payload.x / scale;
          const y = payload.y / scale;
          localStorage.setItem(POSITION_STORAGE_KEY, JSON.stringify({ x, y }));
        } catch (e) {
          console.error("Save overlay position failed:", e);
        }
      }, POSITION_SAVE_DEBOUNCE_MS);
    });
    return () => {
      unlistenP.then((f) => f()).catch(() => {});
      if (timer) window.clearTimeout(timer);
    };
  }, []);

  // Snap-to-center on Cal entry; restore the prior overlay position on exit.
  // Size is also handled here (in addition to the auto-resize observer) so the
  // window expands immediately rather than waiting on the next layout cycle.
  const savedOverlayPosRef = useRef<LogicalPosition | null>(null);
  const prevViewRef = useRef<View>("run");
  useEffect(() => {
    const win = getCurrentWindow();
    const prev = prevViewRef.current;
    prevViewRef.current = view;
    if (prev === view) return;

    if (view === "calibration") {
      (async () => {
        try {
          const physicalPos = await win.outerPosition();
          const monitor = await currentMonitor();
          const scale = monitor?.scaleFactor ?? 1;
          // Save the overlay's position in logical px so we can restore it later.
          savedOverlayPosRef.current = new LogicalPosition(
            physicalPos.x / scale,
            physicalPos.y / scale,
          );

          if (monitor) {
            const monitorLogicalW = monitor.size.width / scale;
            const monitorLogicalX = monitor.position.x / scale;
            const monitorLogicalY = monitor.position.y / scale;
            const x = monitorLogicalX + Math.max(0, (monitorLogicalW - CAL_WIN_W) / 2);
            const y = monitorLogicalY + CAL_TOP_MARGIN;
            await win.setSize(new LogicalSize(CAL_WIN_W, CAL_WIN_H));
            await win.setPosition(new LogicalPosition(x, y));
            // Sync the ResizeObserver's last-size cache so it doesn't fight us.
            lastSizeRef.current = { w: CAL_WIN_W, h: CAL_WIN_H };
          }
        } catch (e) {
          console.error("Cal snap failed:", e);
        }
      })();
    } else if (prev === "calibration" && savedOverlayPosRef.current) {
      // Leaving Cal — restore overlay position. Size is handled by auto-resize.
      const target = savedOverlayPosRef.current;
      win.setPosition(target).catch((e) => console.error("Restore pos failed:", e));
      savedOverlayPosRef.current = null;
    }
  }, [view]);

  // Resize the OS window to match the rendered content. Without this, the
  // window stays at its initial 320x480 and the empty area below the visible
  // box still catches mouse events instead of passing them to the game.
  const lastSizeRef = useRef<{ w: number; h: number } | null>(null);
  const applySize = useCallback((el: HTMLElement) => {
    const rect = el.getBoundingClientRect();
    const w = Math.max(MIN_W, Math.ceil(rect.width));
    const h = Math.max(MIN_H, Math.ceil(rect.height));
    const last = lastSizeRef.current;
    if (last && last.w === w && last.h === h) return;
    lastSizeRef.current = { w, h };
    getCurrentWindow().setSize(new LogicalSize(w, h)).catch((e) => {
      console.error("setSize failed:", e);
    });
  }, []);

  const contentRef = useCallback((el: HTMLDivElement | null) => {
    if (!el) return;
    applySize(el);
    const ro = new ResizeObserver(() => applySize(el));
    ro.observe(el);
    // Cleanup happens implicitly when the next ref callback fires with a new el.
    // We can't return a cleanup from a ref callback in React 18; observers are
    // tied to the element lifetime so this is acceptable.
    (el as unknown as { __ro?: ResizeObserver }).__ro?.disconnect();
    (el as unknown as { __ro?: ResizeObserver }).__ro = ro;
  }, [applySize]);

  const handleSelectBuildOrder = async (path: string) => {
    try {
      const bo = await invoke<BuildOrder>("load_build_order_cmd", { path });
      setBuildOrder(bo);
      setView("run");
    } catch (e) {
      console.error("Failed to load build order:", e);
    }
  };

  if (peek) {
    return (
      <div className="app">
        <div
          ref={contentRef}
          className="app-content"
          style={{ background: "transparent", boxShadow: "none" }}
        >
          <PeekOverlay buildOrder={buildOrder} onExpand={() => setPeek(false)} />
        </div>
      </div>
    );
  }

  const wide = view === "calibration";

  return (
    <div className="app">
      <div ref={contentRef} className={`app-content${wide ? " wide" : ""}`}>
        <div className="titlebar" data-tauri-drag-region>
          <span className="drag-dots"><span /><span /><span /></span>
          <span className="brand"><span className="dot" />Age Up</span>
          <div className="seg">
            <button className={view === "run" ? "on" : ""} onClick={() => setView("run")}>Run</button>
            <button className={view === "library" ? "on" : ""} onClick={() => setView("library")}>Lib</button>
            <button className={view === "calibration" ? "on" : ""} onClick={() => setView("calibration")}>Cal</button>
            <button className={view === "settings" ? "on" : ""} onClick={() => setView("settings")}>Set</button>
          </div>
          <button className="icon-btn" title="Collapse to peek" onClick={() => setPeek(true)}>−</button>
          <button className="icon-btn danger" title="Close" onClick={() => getCurrentWindow().close()}>✕</button>
        </div>
        <div className="view-body">
          {view === "run" && (
            <Overlay onOpenLibrary={() => setView("library")} buildOrder={buildOrder} />
          )}
          {view === "library" && <BuildOrderLibrary onSelect={handleSelectBuildOrder} activeId={buildOrder?.id} />}
          {view === "calibration" && <CalibrationWizard />}
          {view === "settings" && <Settings />}
        </div>
      </div>
    </div>
  );
}

export default App;
