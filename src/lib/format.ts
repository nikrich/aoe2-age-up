import type { Trigger } from "./types";

/** Format a duration in seconds as MM:SS. */
export function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${m}:${String(s).padStart(2, "0")}`;
}

/**
 * Format a game time for display. Returns "—" for nullish or implausible values
 * (game time should never exceed ~3 hours; OCR garbage routinely produces 8-digit numbers).
 */
const MAX_PLAUSIBLE_GAME_TIME_SECONDS = 3 * 60 * 60;

export function formatGameTime(seconds: number | null | undefined): string {
  if (seconds == null || seconds < 0 || seconds > MAX_PLAUSIBLE_GAME_TIME_SECONDS) return "—";
  return formatDuration(seconds);
}

/**
 * Render a Trigger as a short display string for the overlay.
 * Time triggers become "MM:SS"; villager triggers become "@N";
 * resource-only triggers fall back to a compact resource hint.
 */
export function formatTrigger(t: Trigger | undefined | null): string {
  if (!t) return "";
  if (t.time_seconds != null) return formatDuration(t.time_seconds);
  if (t.villagers != null) return `@${t.villagers}`;
  if (t.population_min != null) return `pop ${t.population_min}`;
  if (t.food_min != null) return `F ${t.food_min}`;
  if (t.gold_min != null) return `G ${t.gold_min}`;
  if (t.wood_min != null) return `W ${t.wood_min}`;
  if (t.stone_min != null) return `S ${t.stone_min}`;
  return "";
}

/** Compute elapsed-vs-target delta. Returns null when either value is missing. */
export function computeDelta(
  elapsed: number | null | undefined,
  target: number | null | undefined,
): { sign: "+" | "-"; text: string; behind: boolean } | null {
  if (elapsed == null || target == null) return null;
  const diff = elapsed - target;
  if (diff === 0) return { sign: "+", text: "0:00", behind: false };
  const behind = diff > 0;
  const abs = Math.abs(diff);
  return {
    sign: behind ? "+" : "-",
    text: formatDuration(abs),
    behind,
  };
}
