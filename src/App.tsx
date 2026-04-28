import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Overlay } from "./components/Overlay";
import { BuildOrderLibrary } from "./components/BuildOrderLibrary";
import { Settings } from "./components/Settings";
import "./styles/overlay.css";

type View = "overlay" | "library" | "settings";

function App() {
  const [view, setView] = useState<View>("overlay");

  const handleSelectBuildOrder = async (path: string) => {
    try {
      await invoke("load_build_order_cmd", { path });
      setView("overlay");
    } catch (e) {
      console.error("Failed to load build order:", e);
    }
  };

  return (
    <div className="app">
      <div className="view-switcher">
        <button className={`view-btn ${view === "overlay" ? "view-btn--active" : ""}`} onClick={() => setView("overlay")}>Overlay</button>
        <button className={`view-btn ${view === "library" ? "view-btn--active" : ""}`} onClick={() => setView("library")}>Library</button>
        <button className={`view-btn ${view === "settings" ? "view-btn--active" : ""}`} onClick={() => setView("settings")}>Settings</button>
      </div>
      {view === "overlay" && <Overlay onOpenLibrary={() => setView("library")} />}
      {view === "library" && <BuildOrderLibrary onSelect={handleSelectBuildOrder} />}
      {view === "settings" && <Settings />}
    </div>
  );
}

export default App;
