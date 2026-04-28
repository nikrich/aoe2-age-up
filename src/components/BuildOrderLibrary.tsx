import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { BuildOrderMeta } from "../lib/types";

interface BuildOrderLibraryProps {
  onSelect: (path: string) => void;
}

export function BuildOrderLibrary({ onSelect }: BuildOrderLibraryProps) {
  const [buildOrders, setBuildOrders] = useState<BuildOrderMeta[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<BuildOrderMeta[]>("list_build_orders_cmd")
      .then(setBuildOrders)
      .catch((e) => setError(String(e)));
  }, []);

  if (error) {
    return (
      <div className="library">
        <div className="library-header">Build Orders</div>
        <div style={{ color: "var(--text-secondary)" }}>Error: {error}</div>
      </div>
    );
  }

  return (
    <div className="library">
      <div className="library-header">Build Orders</div>
      {buildOrders.length === 0 && <div style={{ color: "var(--text-secondary)" }}>No build orders found.</div>}
      {buildOrders.map((bo) => (
        <div key={bo.id} className="library-item" onClick={() => onSelect(bo.path)}>
          <div className="library-item-name">{bo.name}</div>
          <div className="library-item-civ">{bo.civilization}</div>
          {bo.description && <div className="library-item-desc">{bo.description}</div>}
          {bo.tags.length > 0 && (
            <div className="library-item-tags">
              {bo.tags.map((tag) => <span key={tag} className="tag">{tag}</span>)}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
