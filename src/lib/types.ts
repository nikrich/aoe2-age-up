export type Difficulty = "beginner" | "intermediate" | "advanced";
export type Phase = "dark" | "feudal" | "castle" | "imperial";

export interface BuildOrder {
  id: string;
  name: string;
  civilization: string;
  author?: string;
  description?: string;
  source_url?: string;
  tags: string[];
  difficulty?: Difficulty;
  glyph?: string;
  steps: Step[];
}

export interface Step {
  action: string;
  at: Trigger;
  notes?: string;
  villagers_assigned?: VillagerAssignment;
  target_time_seconds?: number;
  phase?: Phase;
}

export interface Trigger {
  time_seconds?: number;
  villagers?: number;
  population_min?: number;
  food_min?: number;
  wood_min?: number;
  gold_min?: number;
  stone_min?: number;
  mode: "All" | "Any";
}

export interface VillagerAssignment {
  food: number;
  wood: number;
  gold: number;
  stone: number;
  idle: number;
}

export interface BuildOrderMeta {
  id: string;
  name: string;
  civilization: string;
  tags: string[];
  description?: string;
  difficulty?: Difficulty;
  glyph?: string;
  path: string;
}

export interface StepChangedPayload {
  index: number;
  step: Step;
  total: number;
}

export interface GameState {
  food?: number;
  wood?: number;
  gold?: number;
  stone?: number;
  villagers?: number;
  population?: [number, number];
  game_time_seconds?: number;
}

export interface Settings {
  capture_interval_ms: number;
  auto_advance: boolean;
  click_through: boolean;
  overlay_opacity: number;
  hotkeys: HotkeyConfig;
  ocr_backend: "Template" | "Tesseract";
}

export interface HotkeyConfig {
  advance_step: string;
  previous_step: string;
  reset: string;
  pause_capture: string;
  toggle_visibility: string;
  toggle_click_through: string;
}

export interface Calibration {
  profile_name: string;
  resolution: [number, number];
  ui_scale: number;
  regions: Record<string, Region>;
}

export interface Region {
  x: number;
  y: number;
  width: number;
  height: number;
}
