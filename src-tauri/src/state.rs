use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::oneshot;

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

impl Default for Calibration {
    fn default() -> Self {
        Self {
            profile_name: "default".to_string(),
            resolution: (1920, 1080),
            ui_scale: 1.0,
            regions: HashMap::new(),
        }
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
            capture_running: false,
            calibration: Calibration::default(),
            settings: Settings::default(),
        }
    }
}

/// Handle for controlling the capture loop from IPC commands.
pub struct CaptureHandle {
    pub stop_tx: Option<oneshot::Sender<()>>,
}
