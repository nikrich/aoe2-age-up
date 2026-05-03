import { useState } from "react";
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
  );
}

export default App;
