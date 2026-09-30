use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::build_order::parser::load_build_order;
use crate::build_order::{BuildOrder, BuildOrderMeta, Step};
use crate::error::AppError;
use crate::capture::fallback::XcapCapture;
use crate::capture::loop_task::spawn_capture_loop;
use crate::capture::CaptureBackend;
use crate::ocr::windows_ocr::TesseractPipeline;
use crate::state::{AppState, Calibration, CaptureHandle, OcrBackend, RegionKind, Settings};
use crate::storage::Storage;

#[derive(Clone, Serialize)]
pub struct StepChangedPayload {
    pub index: usize,
    pub step: Step,
    pub total: usize,
}

pub fn emit_step_changed(app: &AppHandle, state: &AppState) {
    if let Some(ref bo) = state.current_build_order {
        if state.current_step_index < bo.steps.len() {
            let payload = StepChangedPayload {
                index: state.current_step_index,
                step: bo.steps[state.current_step_index].clone(),
                total: bo.steps.len(),
            };
            let _ = app.emit("step-changed", payload);
        }
    }
}

#[tauri::command]
pub fn load_build_order_cmd(
    path: String,
    app: AppHandle,
    state: State<Mutex<AppState>>,
) -> Result<BuildOrder, AppError> {
    let bo = load_build_order(std::path::Path::new(&path))
        .map_err(|e| AppError::Storage(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    let mut s = state.lock().unwrap();
    s.current_build_order = Some(bo.clone());
    s.current_step_index = 0;
    s.peak_game_state = crate::state::GameState::default();
    emit_step_changed(&app, &s);

    Ok(bo)
}

#[tauri::command]
pub fn list_build_orders_cmd(
    storage: State<Storage>,
) -> Result<Vec<BuildOrderMeta>, AppError> {
    storage
        .list_build_orders()
        .map_err(|e| AppError::Storage(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))
}

#[tauri::command]
pub fn advance_step(
    app: AppHandle,
    state: State<Mutex<AppState>>,
) -> Result<usize, AppError> {
    let mut s = state.lock().unwrap();
    let bo = s.current_build_order.as_ref().ok_or(AppError::NoBuildOrderLoaded)?;
    if s.current_step_index < bo.steps.len() - 1 {
        s.current_step_index += 1;
    }
    let idx = s.current_step_index;
    emit_step_changed(&app, &s);
    Ok(idx)
}

#[tauri::command]
pub fn previous_step(
    app: AppHandle,
    state: State<Mutex<AppState>>,
) -> Result<usize, AppError> {
    let mut s = state.lock().unwrap();
    let _bo = s.current_build_order.as_ref().ok_or(AppError::NoBuildOrderLoaded)?;
    if s.current_step_index > 0 {
        s.current_step_index -= 1;
    }
    let idx = s.current_step_index;
    emit_step_changed(&app, &s);
    Ok(idx)
}

#[tauri::command]
pub fn reset_steps(
    app: AppHandle,
    state: State<Mutex<AppState>>,
) -> Result<(), AppError> {
    let mut s = state.lock().unwrap();
    s.current_step_index = 0;
    s.peak_game_state = crate::state::GameState::default();
    emit_step_changed(&app, &s);
    Ok(())
}

#[tauri::command]
pub fn get_settings(
    state: State<Mutex<AppState>>,
) -> Result<Settings, AppError> {
    let s = state.lock().unwrap();
    Ok(s.settings.clone())
}

#[tauri::command]
pub fn get_calibration(
    state: State<Mutex<AppState>>,
) -> Result<Calibration, AppError> {
    let s = state.lock().unwrap();
    Ok(s.calibration.clone())
}

/// Locate the Tesseract binary, its dependency directory and its tessdata
/// directory, preferring the bundled sidecar (Windows installer) and falling
/// back to a system install (Linux/macOS dev machines).
pub fn resolve_tesseract(
    exe_dir: &std::path::Path,
    resource_dir: &std::path::Path,
) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    #[cfg(target_os = "windows")]
    let sidecar_names = [
        "tesseract-x86_64-pc-windows-msvc.exe",
        "tesseract.exe",
    ];
    #[cfg(not(target_os = "windows"))]
    let sidecar_names = [
        "tesseract-x86_64-unknown-linux-gnu",
        "tesseract-aarch64-unknown-linux-gnu",
        "tesseract",
    ];

    // 1. Bundled sidecar next to the executable
    let mut tesseract_path = None;
    for name in sidecar_names {
        let candidate = exe_dir.join(name);
        if candidate.exists() {
            tesseract_path = Some(candidate);
            break;
        }
    }

    // 2. Bundled dependency/tessdata directories (Windows bundle layout)
    let deps_dir = {
        let in_resource = resource_dir.join("tesseract-deps");
        let in_exe = exe_dir.join("tesseract-deps");
        if in_resource.is_dir() {
            in_resource
        } else if in_exe.is_dir() {
            in_exe
        } else {
            std::path::PathBuf::new() // no bundled deps (Linux dev machine)
        }
    };

    // 3. System tesseract on PATH (Linux/macOS)
    if tesseract_path.is_none() {
        tesseract_path = find_in_path("tesseract");
    }

    // 4. Common system locations as a last resort
    if tesseract_path.is_none() {
        for candidate in ["/usr/bin/tesseract", "/usr/local/bin/tesseract", "/bin/tesseract"] {
            let p = std::path::PathBuf::from(candidate);
            if p.exists() {
                tesseract_path = Some(p);
                break;
            }
        }
    }

    let tesseract_path = tesseract_path.unwrap_or_else(|| std::path::PathBuf::from("tesseract"));

    // 5. Tessdata: bundled first, then distro locations, then TESSDATA_PREFIX.
    //    If nothing is found we hand an empty path to the pipeline, which then
    //    lets tesseract use its own compiled-in default.
    let mut tessdata_path = deps_dir.join("tessdata");
    if !tessdata_path.is_dir() {
        let mut found: Option<std::path::PathBuf> = None;
        if let Ok(prefix) = std::env::var("TESSDATA_PREFIX") {
            let p = std::path::PathBuf::from(&prefix);
            // Accept both a dir containing eng.traineddata and a parent of tessdata/
            if p.join("eng.traineddata").exists() {
                found = Some(p);
            } else if p.join("tessdata").is_dir() {
                found = Some(p.join("tessdata"));
            }
        }
        if found.is_none() {
            for candidate in [
                "/usr/share/tessdata",
                "/usr/share/tesseract-ocr/5/tessdata",
                "/usr/share/tesseract-ocr/4.00/tessdata",
                "/usr/local/share/tessdata",
            ] {
                let p = std::path::PathBuf::from(candidate);
                if p.is_dir() {
                    found = Some(p);
                    break;
                }
            }
        }
        if let Some(p) = found {
            tessdata_path = p;
        }
    }

    (tesseract_path, deps_dir, tessdata_path)
}

/// Look up an executable in PATH without shelling out to `which`.
fn find_in_path(name: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

#[tauri::command]
pub fn start_capture(
    app: AppHandle,
    state: State<Mutex<AppState>>,
    capture_handle: State<Mutex<CaptureHandle>>,
) -> Result<(), AppError> {
    {
        let mut s = state.lock().unwrap();
        if s.capture_running {
            return Ok(()); // Already running
        }
        s.capture_running = true;
    }

    // Wrap in a closure so we can reset capture_running on any failure
    let result = (|| -> Result<(), AppError> {
        let backend = Box::new(
            XcapCapture::new().map_err(|e| AppError::Capture(e.to_string()))?
        );

        let exe_dir = std::env::current_exe()
            .map_err(|e| AppError::Capture(format!("Failed to get exe path: {}", e)))?
            .parent()
            .unwrap()
            .to_path_buf();
        let resource_dir = app.path().resource_dir()
            .map_err(|e| AppError::Capture(format!("Failed to resolve resource dir: {}", e)))?;

        let (tesseract_path, deps_dir, tessdata_path) =
            resolve_tesseract(&exe_dir, &resource_dir);

        let ocr_backend = {
            let s = state.lock().unwrap();
            s.settings.ocr_backend.clone()
        };

        let ocr: Box<dyn crate::ocr::OcrPipeline> = match ocr_backend {
            // Pure-Rust template matching over synthetic glyphs: no external
            // binary needed, but only meant as a last-resort backend.
            OcrBackend::Template => {
                tracing::warn!("Using the template OCR backend (synthetic glyphs, low accuracy)");
                Box::new(crate::ocr::TemplatePipeline::new(
                    crate::ocr::fixture::generate_fixture_templates(),
                ))
            }
            OcrBackend::Tesseract => {
                tracing::info!(
                    "Tesseract: {:?}, tessdata: {:?}, deps: {:?}",
                    tesseract_path,
                    tessdata_path,
                    deps_dir
                );
                Box::new(
                    TesseractPipeline::new(tesseract_path, tessdata_path, deps_dir)
                        .map_err(|e| AppError::Capture(e.to_string()))?,
                )
            }
        };

        let stop_tx = spawn_capture_loop(app, backend, ocr);

        {
            let mut handle = capture_handle.lock().unwrap();
            handle.stop_tx = Some(stop_tx);
        }

        Ok(())
    })();

    if result.is_err() {
        let mut s = state.lock().unwrap();
        s.capture_running = false;
    }

    result
}

#[tauri::command]
pub fn stop_capture(
    state: State<Mutex<AppState>>,
    capture_handle: State<Mutex<CaptureHandle>>,
) -> Result<(), AppError> {
    {
        let mut s = state.lock().unwrap();
        s.capture_running = false;
    }

    let mut handle = capture_handle.lock().unwrap();
    if let Some(tx) = handle.stop_tx.take() {
        let _ = tx.send(());
    }

    Ok(())
}

/// Capture a clean screenshot (no rectangles drawn) and return it as a base64
/// data URI for the frontend to use as a calibration canvas backdrop.
#[tauri::command]
pub fn capture_calibration_screenshot() -> Result<String, AppError> {
    use base64::Engine;

    let mut backend = XcapCapture::new()
        .map_err(|e| AppError::Capture(e.to_string()))?;
    let frame = backend.next_frame()
        .map_err(|e| AppError::Capture(e.to_string()))?;

    let img = image::RgbaImage::from_raw(frame.width, frame.height, frame.data)
        .ok_or_else(|| AppError::Capture("Invalid frame data".to_string()))?;

    let mut buf: Vec<u8> = Vec::with_capacity(1 << 20);
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .map_err(|e| AppError::Capture(format!("Failed to encode PNG: {}", e)))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
    Ok(format!("data:image/png;base64,{}", b64))
}

/// Replace the active calibration with the supplied one and persist it.
/// The capture loop reads regions per frame, so updates apply on the next OCR cycle.
#[tauri::command]
pub fn set_calibration(
    calibration: Calibration,
    state: State<Mutex<AppState>>,
    storage: State<Storage>,
) -> Result<(), AppError> {
    {
        let mut s = state.lock().unwrap();
        s.calibration = calibration.clone();
    }
    storage
        .save_calibration(&calibration)
        .map_err(|e| AppError::Storage(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
    Ok(())
}

#[tauri::command]
pub fn generate_calibration_image(
    state: State<Mutex<AppState>>,
) -> Result<String, AppError> {
    let regions = {
        let s = state.lock().unwrap();
        s.calibration.regions.clone()
    };

    let mut backend = XcapCapture::new()
        .map_err(|e| AppError::Capture(e.to_string()))?;
    let frame = backend.next_frame()
        .map_err(|e| AppError::Capture(e.to_string()))?;

    let mut img = image::RgbaImage::from_raw(frame.width, frame.height, frame.data)
        .ok_or_else(|| AppError::Capture("Invalid frame data".to_string()))?;

    // Draw rectangles for each calibration region
    let colors: Vec<(RegionKind, [u8; 4])> = vec![
        (RegionKind::Food, [255, 0, 0, 255]),       // Red
        (RegionKind::Wood, [0, 255, 0, 255]),        // Green
        (RegionKind::Gold, [255, 255, 0, 255]),      // Yellow
        (RegionKind::Stone, [128, 128, 128, 255]),   // Gray
        (RegionKind::Villagers, [0, 255, 255, 255]), // Cyan
        (RegionKind::Population, [255, 0, 255, 255]),// Magenta
        (RegionKind::GameTime, [255, 128, 0, 255]),  // Orange
    ];

    for (kind, color) in &colors {
        if let Some(region) = regions.get(kind) {
            let pixel = image::Rgba(*color);
            let x1 = region.x;
            let y1 = region.y;
            let x2 = (region.x + region.width).min(frame.width - 1);
            let y2 = (region.y + region.height).min(frame.height - 1);

            // Draw top and bottom edges
            for x in x1..=x2 {
                img.put_pixel(x, y1, pixel);
                img.put_pixel(x, y2, pixel);
            }
            // Draw left and right edges
            for y in y1..=y2 {
                img.put_pixel(x1, y, pixel);
                img.put_pixel(x2, y, pixel);
            }
        }
    }

    let out_path = std::env::temp_dir().join("aoe_calibration.png");
    img.save(&out_path)
        .map_err(|e| AppError::Capture(format!("Failed to save calibration image: {}", e)))?;

    let path_str = out_path.to_string_lossy().to_string();
    tracing::info!("Calibration image saved to {}", path_str);
    Ok(path_str)
}
