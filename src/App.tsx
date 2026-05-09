import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";
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

  return (
    <div className="app">
      <div ref={contentRef} className="app-content">
        <div className="titlebar" data-tauri-drag-region>
          <span className="drag-dots"><span /><span /><span /></span>
          <span className="brand"><span className="dot" />Open Age</span>
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
