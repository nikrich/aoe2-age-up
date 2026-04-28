import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Settings as SettingsType } from "../lib/types";

export function Settings() {
  const [settings, setSettings] = useState<SettingsType | null>(null);

  useEffect(() => {
    invoke<SettingsType>("get_settings").then(setSettings).catch(console.error);
  }, []);

  if (!settings) {
    return (
      <div className="settings">
        <div className="settings-header">Settings</div>
        <div style={{ color: "var(--text-secondary)" }}>Loading...</div>
      </div>
    );
  }

  const hotkeys = settings.hotkeys;

  return (
    <div className="settings">
      <div className="settings-header">Hotkeys</div>
      <div className="settings-row"><span className="settings-label">Advance step</span><span className="settings-value">{hotkeys.advance_step}</span></div>
      <div className="settings-row"><span className="settings-label">Previous step</span><span className="settings-value">{hotkeys.previous_step}</span></div>
      <div className="settings-row"><span className="settings-label">Reset</span><span className="settings-value">{hotkeys.reset}</span></div>
      <div className="settings-row"><span className="settings-label">Pause capture</span><span className="settings-value">{hotkeys.pause_capture}</span></div>
      <div className="settings-row"><span className="settings-label">Toggle overlay</span><span className="settings-value">{hotkeys.toggle_visibility}</span></div>
      <div className="settings-row"><span className="settings-label">Toggle click-through</span><span className="settings-value">{hotkeys.toggle_click_through}</span></div>
    </div>
  );
}
