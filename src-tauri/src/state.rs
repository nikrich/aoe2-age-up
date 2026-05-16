use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use std::sync::mpsc;

use crate::build_order::BuildOrder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub food: Option<u32>,
    pub wood: Option<u32>,
    pub gold: Option<u32>,
    pub stone: Option<u32>,
    pub villagers: Option<u32>,
    pub population: Option<(u32, u32)>,
    pub game_time_seconds: Option<u32>,
    #[serde(skip)]
    pub last_updated: Option<Instant>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            food: None,
            wood: None,
            gold: None,
            stone: None,
            villagers: None,
            population: None,
            game_time_seconds: None,
            last_updated: None,
        }
    }
}

/// Implausible game time (>3h) is treated as garbage OCR and ignored when
/// updating the peak. Matches the cap used by the frontend overlay.
const MAX_PLAUSIBLE_GAME_TIME_SECONDS: u32 = 3 * 60 * 60;

impl GameState {
    /// Merge `other` into `self`, keeping the per-field maximum. Used to
    /// build a session "peak" state that latches the highest value each
    /// resource / villager count / game time has ever cleanly hit, so a
    /// single transient OCR misread can't undo a satisfied trigger.
    pub fn merge_max(&mut self, other: &GameState) {
        self.food = max_opt(self.food, other.food);
        self.wood = max_opt(self.wood, other.wood);
        self.gold = max_opt(self.gold, other.gold);
        self.stone = max_opt(self.stone, other.stone);
        self.villagers = max_opt(self.villagers, other.villagers);
        self.population = match (self.population, other.population) {
            (Some((a, am)), Some((b, bm))) => Some((a.max(b), am.max(bm))),
            (Some(p), None) | (None, Some(p)) => Some(p),
            (None, None) => None,
        };
        let sane_time = other.game_time_seconds.filter(|t| *t <= MAX_PLAUSIBLE_GAME_TIME_SECONDS);
        self.game_time_seconds = max_opt(self.game_time_seconds, sane_time);
    }
}

fn max_opt(a: Option<u32>, b: Option<u32>) -> Option<u32> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.max(y)),
        (Some(x), None) | (None, Some(x)) => Some(x),
        (None, None) => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, Copy)]
pub enum RegionKind {
    Food,
    Wood,
    Gold,
    Stone,
    Villagers,
    Population,
    GameTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calibration {
    pub profile_name: String,
    pub resolution: (u32, u32),
    pub ui_scale: f32,
    pub regions: HashMap<RegionKind, Region>,
}

impl Calibration {
    /// Default calibration for AoE2:DE at 1920x1080 resolution.
    /// These are approximate regions for the standard UI layout.
    pub fn default_1080p() -> Self {
        let mut regions = HashMap::new();
        // Resource bar positions for 1080p AoE2:DE
        // These coordinates worked in log-tess (Food=450, Wood=396 correct)
        regions.insert(RegionKind::Wood, Region { x: 50, y: 20, width: 48, height: 20 });
        regions.insert(RegionKind::Food, Region { x: 143, y: 20, width: 48, height: 20 });
        regions.insert(RegionKind::Gold, Region { x: 243, y: 20, width: 48, height: 20 });
        regions.insert(RegionKind::Stone, Region { x: 338, y: 20, width: 48, height: 20 });
        regions.insert(RegionKind::Villagers, Region { x: 420, y: 30, width: 28, height: 18 });
        regions.insert(RegionKind::Population, Region { x: 450, y: 20, width: 55, height: 20 });
        regions.insert(RegionKind::GameTime, Region { x: 830, y: 2, width: 90, height: 22 });

        Self {
            profile_name: "1080p-default".to_string(),
            resolution: (1920, 1080),
            ui_scale: 1.0,
            regions,
        }
    }
}

impl Default for Calibration {
    fn default() -> Self {
        Self::default_1080p()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub advance_step: String,
    pub previous_step: String,
    pub reset: String,
    pub pause_capture: String,
    pub toggle_visibility: String,
    pub toggle_click_through: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            advance_step: "Ctrl+Alt+Right".to_string(),
            previous_step: "Ctrl+Alt+Left".to_string(),
            reset: "Ctrl+Alt+R".to_string(),
            pause_capture: "Ctrl+Alt+P".to_string(),
            toggle_visibility: "Ctrl+Alt+H".to_string(),
            toggle_click_through: "Ctrl+Alt+C".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OcrBackend {
    Template,
    Tesseract,
}

impl Default for OcrBackend {
    fn default() -> Self {
        Self::Template
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub capture_interval_ms: u64,
    pub auto_advance: bool,
    pub click_through: bool,
    pub overlay_opacity: f32,
    pub hotkeys: HotkeyConfig,
    pub ocr_backend: OcrBackend,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            capture_interval_ms: 1000,
            auto_advance: true,
            click_through: false,
            overlay_opacity: 0.9,
            hotkeys: HotkeyConfig::default(),
            ocr_backend: OcrBackend::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub current_build_order: Option<BuildOrder>,
    pub current_step_index: usize,
    pub last_game_state: GameState,
    /// Per-field max of every clean OCR reading this session. Auto-advance
    /// evaluates triggers against this so transient misreads (vill 21 read
    /// once, then OCR drops back to 13) can't un-satisfy a trigger.
    /// Reset whenever a build order is loaded or steps are reset.
    pub peak_game_state: GameState,
    pub capture_running: bool,
    pub calibration: Calibration,
    pub settings: Settings,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_build_order: None,
            current_step_index: 0,
            last_game_state: GameState::default(),
            peak_game_state: GameState::default(),
            capture_running: false,
            calibration: Calibration::default(),
            settings: Settings::default(),
        }
    }
}

/// Handle for controlling the capture loop from IPC commands.
pub struct CaptureHandle {
    pub stop_tx: Option<mpsc::Sender<()>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_max_keeps_higher_per_field() {
        let mut peak = GameState::default();
        peak.merge_max(&GameState {
            villagers: Some(21),
            food: Some(200),
            game_time_seconds: Some(540),
            ..GameState::default()
        });
        // Now an OCR misread comes in: villager count "drops" to 13, food drops, etc.
        peak.merge_max(&GameState {
            villagers: Some(13),
            food: Some(50),
            game_time_seconds: Some(560),
            ..GameState::default()
        });
        assert_eq!(peak.villagers, Some(21));
        assert_eq!(peak.food, Some(200));
        assert_eq!(peak.game_time_seconds, Some(560));
    }

    #[test]
    fn merge_max_rejects_garbage_game_time() {
        let mut peak = GameState::default();
        peak.merge_max(&GameState {
            game_time_seconds: Some(600),
            ..GameState::default()
        });
        // OCR misread: 12 million seconds. Must not latch.
        peak.merge_max(&GameState {
            game_time_seconds: Some(12_006_841),
            ..GameState::default()
        });
        assert_eq!(peak.game_time_seconds, Some(600));
    }

    #[test]
    fn merge_max_handles_none_gracefully() {
        let mut peak = GameState {
            villagers: Some(10),
            ..GameState::default()
        };
        peak.merge_max(&GameState::default());
        assert_eq!(peak.villagers, Some(10));
    }
}
