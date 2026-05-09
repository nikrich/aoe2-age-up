import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { BuildOrderMeta, Difficulty } from "../lib/types";

interface BuildOrderLibraryProps {
  onSelect: (path: string) => void;
  activeId?: string;
}

type DiffFilter = "all" | Difficulty;

function deriveGlyph(meta: BuildOrderMeta): string {
  if (meta.glyph) return meta.glyph;
  // Take first letters of words in the name as a fallback (e.g. "Fast Castle" → "FC")
  const words = meta.name.split(/[\s—-]+/).filter(Boolean);
  return words.slice(0, 3).map((w) => w[0]?.toUpperCase() ?? "").join("") || "BO";
}

export function BuildOrderLibrary({ onSelect, activeId }: BuildOrderLibraryProps) {
  const [buildOrders, setBuildOrders] = useState<BuildOrderMeta[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [diffFilter, setDiffFilter] = useState<DiffFilter>("all");

  useEffect(() => {
    invoke<BuildOrderMeta[]>("list_build_orders_cmd")
      .then(setBuildOrders)
      .catch((e) => setError(String(e)));
  }, []);

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    return buildOrders.filter((bo) => {
      if (diffFilter !== "all" && bo.difficulty !== diffFilter) return false;
      if (!q) return true;
      const hay = [
        bo.name,
        bo.civilization,
        bo.description ?? "",
        ...bo.tags,
      ].join(" ").toLowerCase();
      return hay.includes(q);
    });
  }, [buildOrders, search, diffFilter]);

  return (
    <div className="library">
      <div className="lib-search">
        <input
          type="text"
          placeholder="Search build orders…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <span className="key">/</span>
      </div>

      <div className="lib-filters">
        <button
          className={`filter-pill ${diffFilter === "all" ? "on" : ""}`}
          onClick={() => setDiffFilter("all")}
        >
          ALL
        </button>
        <button
          className={`filter-pill ${diffFilter === "beginner" ? "on" : ""}`}
          onClick={() => setDiffFilter("beginner")}
        >
          BEG
        </button>
        <button
          className={`filter-pill ${diffFilter === "intermediate" ? "on" : ""}`}
          onClick={() => setDiffFilter("intermediate")}
        >
          INT
        </button>
        <button
          className={`filter-pill ${diffFilter === "advanced" ? "on" : ""}`}
          onClick={() => setDiffFilter("advanced")}
        >
          ADV
        </button>
      </div>

      {error && <div className="lib-error">Error: {error}</div>}

      <div className="lib-list">
        {!error && filtered.length === 0 && (
          <div className="lib-empty">No build orders match.</div>
        )}
        {filtered.map((bo) => (
          <div
            key={bo.id}
            className={`bo-card ${bo.id === activeId ? "active" : ""}`}
            onClick={() => onSelect(bo.path)}
          >
            <div className="glyph">{deriveGlyph(bo)}</div>
            <div className="info">
              <div className="top">
                <span className="name">{bo.name}</span>
                <span className="civ">{bo.civilization}</span>
                {bo.difficulty && (
                  <span className={`diff ${bo.difficulty}`}>{bo.difficulty.slice(0, 3)}</span>
                )}
              </div>
              {bo.description && <div className="desc">{bo.description}</div>}
              {bo.tags.length > 0 && (
                <div className="tags">
                  {bo.tags.slice(0, 6).map((tag) => (
                    <span key={tag} className="tag">{tag}</span>
                  ))}
                </div>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
