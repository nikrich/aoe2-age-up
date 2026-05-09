import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Overlay } from "./components/Overlay";
import { BuildOrderLibrary } from "./components/BuildOrderLibrary";
import { Settings } from "./components/Settings";
import type { BuildOrder } from "./lib/types";
import "./styles/overlay.css";

type View = "overlay" | "library" | "settings";

function App() {
  const [view, setView] = useState<View>("overlay");
  const [buildOrder, setBuildOrder] = useState<BuildOrder | null>(null);
  const contentRef = useRef<HTMLDivElement>(null);

  // Make transparent areas click-through at the OS level
  useEffect(() => {
    const appWindow = getCurrentWindow();
    // Start with click-through enabled
    appWindow.setIgnoreCursorEvents(true);

    const handleMouseEnter = () => appWindow.setIgnoreCursorEvents(false);
    const handleMouseLeave = () => appWindow.setIgnoreCursorEvents(true);

    const el = contentRef.current;
    if (el) {
      el.addEventListener("mouseenter", handleMouseEnter);
      el.addEventListener("mouseleave", handleMouseLeave);
    }

    return () => {
      if (el) {
        el.removeEventListener("mouseenter", handleMouseEnter);
        el.removeEventListener("mouseleave", handleMouseLeave);
      }
    };
  }, []);

  const handleSelectBuildOrder = async (path: string) => {
    try {
      const bo = await invoke<BuildOrder>("load_build_order_cmd", { path });
      setBuildOrder(bo);
      setView("overlay");
    } catch (e) {
      console.error("Failed to load build order:", e);
    }
  };

  return (
    <div className="app">
      <div className="app-content" ref={contentRef}>
        <div className="view-switcher" data-tauri-drag-region>
          <button className={`view-btn ${view === "overlay" ? "view-btn--active" : ""}`} onClick={() => setView("overlay")}>Overlay</button>
          <button className={`view-btn ${view === "library" ? "view-btn--active" : ""}`} onClick={() => setView("library")}>Library</button>
          <button className={`view-btn ${view === "settings" ? "view-btn--active" : ""}`} onClick={() => setView("settings")}>Settings</button>
          <button className="view-btn close-btn" onClick={() => getCurrentWindow().close()}>✕</button>
        </div>
        {view === "overlay" && <Overlay onOpenLibrary={() => setView("library")} buildOrder={buildOrder} />}
        {view === "library" && <BuildOrderLibrary onSelect={handleSelectBuildOrder} />}
        {view === "settings" && <Settings />}
      </div>
    </div>
  );
}

export default App;
