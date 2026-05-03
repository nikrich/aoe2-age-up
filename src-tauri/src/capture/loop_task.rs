use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::Instant;

use tauri::{AppHandle, Emitter, Manager};
use tracing::{debug, info, warn};

use crate::build_order::engine::evaluate;
use crate::ipc::emit_step_changed;
use crate::ocr::preprocess::crop_region;
use crate::ocr::OcrPipeline;
use crate::state::{AppState, GameState, RegionKind};

use super::CaptureBackend;

/// Spawn the capture loop on a dedicated thread. Returns a stop sender.
pub fn spawn_capture_loop(
    app: AppHandle,
    mut backend: Box<dyn CaptureBackend>,
    ocr: Box<dyn OcrPipeline>,
) -> mpsc::Sender<()> {
    let (stop_tx, stop_rx) = mpsc::channel::<()>();

    std::thread::spawn(move || {
        info!("Capture loop started");
        let mut last_hashes: HashMap<RegionKind, u64> = HashMap::new();

        loop {
            // Check for stop signal (non-blocking)
            if stop_rx.try_recv().is_ok() {
                info!("Capture loop received stop signal");
                break;
            }

            let interval_ms = {
                let state_mutex = app.state::<Mutex<AppState>>();
                let s = state_mutex.lock().unwrap();
                if !s.capture_running {
                    break;
                }
                s.settings.capture_interval_ms
            };

            // Capture frame
            let frame = match backend.next_frame() {
                Ok(frame) => frame,
                Err(e) => {
                    warn!("Capture failed: {}", e);
                    std::thread::sleep(std::time::Duration::from_millis(interval_ms));
                    continue;
                }
            };

            // Read calibration regions and process OCR
            let regions = {
                let state_mutex = app.state::<Mutex<AppState>>();
                let s = state_mutex.lock().unwrap();
                s.calibration.regions.clone()
            };

            // Start from previous state so values persist when regions are skipped
            let mut new_game_state = {
                let state_mutex = app.state::<Mutex<AppState>>();
                let s = state_mutex.lock().unwrap();
                s.last_game_state.clone()
            };
            let mut any_update = false;

            debug!("Processing frame {}x{}, {} regions", frame.width, frame.height, regions.len());

            for (kind, region) in &regions {
                let cropped = crop_region(
                    &frame.data,
                    frame.width,
                    frame.height,
                    region.x,
                    region.y,
                    region.width,
                    region.height,
                );

                let cropped = match cropped {
                    Some(c) => c,
                    None => continue,
                };

                // Quick hash to skip unchanged regions
                let hash = quick_hash(cropped.as_raw());
                if last_hashes.get(kind) == Some(&hash) {
                    continue;
                }
                last_hashes.insert(*kind, hash);
                any_update = true;

                let raw = cropped.as_raw();
                let w = cropped.width();
                let h = cropped.height();

                match kind {
                    RegionKind::Food => {
                        new_game_state.food = ocr.read_number(raw, w, h, *kind).unwrap_or(None);
                    }
                    RegionKind::Wood => {
                        new_game_state.wood = ocr.read_number(raw, w, h, *kind).unwrap_or(None);
                    }
                    RegionKind::Gold => {
                        new_game_state.gold = ocr.read_number(raw, w, h, *kind).unwrap_or(None);
                    }
                    RegionKind::Stone => {
                        new_game_state.stone = ocr.read_number(raw, w, h, *kind).unwrap_or(None);
                    }
                    RegionKind::Villagers => {
                        new_game_state.villagers = ocr.read_number(raw, w, h, *kind).unwrap_or(None);
                    }
                    RegionKind::Population => {
                        new_game_state.population = ocr.read_population(raw, w, h).unwrap_or(None);
                    }
                    RegionKind::GameTime => {
                        new_game_state.game_time_seconds = ocr.read_time(raw, w, h).unwrap_or(None);
                    }
                }
            }

            if any_update {
                new_game_state.last_updated = Some(Instant::now());

                // Emit game state to frontend
                let _ = app.emit("game-state", &new_game_state);

                // Evaluate triggers and auto-advance
                let state_mutex = app.state::<Mutex<AppState>>();
                let mut s = state_mutex.lock().unwrap();
                s.last_game_state = new_game_state.clone();

                if s.settings.auto_advance {
                    if let Some(ref bo) = s.current_build_order {
                        if s.current_step_index < bo.steps.len() - 1 {
                            let next_trigger = &bo.steps[s.current_step_index + 1].at;
                            let result = evaluate(next_trigger, &new_game_state);
                            debug!("Auto-advance check: step={}, trigger={:?}, vils={:?}, time={:?}, result={}",
                                s.current_step_index, next_trigger, new_game_state.villagers, new_game_state.game_time_seconds, result);
                            if result {
                                s.current_step_index += 1;
                                emit_step_changed(&app, &s);
                                info!("Auto-advanced to step {}", s.current_step_index);
                            }
                        }
                    } else {
                        debug!("Auto-advance: no build order loaded");
                    }
                } else {
                    debug!("Auto-advance: disabled");
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(interval_ms));
        }

        info!("Capture loop stopped");
    });

    stop_tx
}

fn quick_hash(data: &[u8]) -> u64 {
    // FNV-1a hash for fast comparison
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data.iter().step_by(16) {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
