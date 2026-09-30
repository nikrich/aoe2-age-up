#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::Manager;

mod build_order;
mod capture;
mod error;
mod hotkeys;
mod ipc;
mod ocr;
mod state;
mod storage;

use state::{AppState, CaptureHandle};
use storage::Storage;

fn setup_logging() {
    use tracing_subscriber::fmt::writer::MakeWriterExt;

    // In release mode, log to a file in the user's temp dir so we can diagnose crashes
    if cfg!(not(debug_assertions)) {
        let log_path = std::env::temp_dir().join("aoe-overlay.log");
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_path);

        if let Ok(file) = file {
            tracing_subscriber::fmt()
                .with_env_filter("aoe_overlay=debug")
                .with_writer(std::io::stderr.and(file))
                .with_ansi(false)
                .init();
            return;
        }
    }

    tracing_subscriber::fmt()
        .with_env_filter("aoe_overlay=debug")
        .init();
}

fn setup_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let log_path = std::env::temp_dir().join("aoe-overlay-crash.log");
        let msg = format!("PANIC: {}\nLocation: {:?}\n", info, info.location());
        let _ = std::fs::write(&log_path, &msg);
        // Also append to the main log
        let main_log = std::env::temp_dir().join("aoe-overlay.log");
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&main_log)
            .and_then(|mut f| std::io::Write::write_all(&mut f, msg.as_bytes()));
        default_hook(info);
    }));
}

fn main() {
    // The overlay depends on always-on-top, and GTK's `set_keep_above` is a
    // no-op on native Wayland (KWin exposes no keep-above protocol for
    // xdg_toplevel). AoE2:DE runs through XWayland, so run the GTK window on
    // X11 as well: `_NET_WM_STATE_ABOVE` then stacks it over the fullscreen
    // game window. An explicit GDK_BACKEND from the caller still wins.
    if std::env::var_os("GDK_BACKEND").is_none() {
        std::env::set_var("GDK_BACKEND", "x11");
    }

    setup_panic_hook();
    setup_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            let storage = Storage::new(app_data_dir)?;

            // Tauri maps `../build-orders/*` (the path in tauri.conf.json) to
            // `<resources>/_up_/build-orders`, so both layouts must be probed —
            // otherwise the bundled orders never land in the app data dir and
            // the Library shows up empty.
            let resource_dir = app
                .path()
                .resource_dir()
                .expect("Failed to get resource dir");
            for candidate in [
                resource_dir.join("build-orders"),
                resource_dir.join("_up_").join("build-orders"),
            ] {
                if candidate.is_dir() {
                    if let Err(e) = storage.copy_bundled_build_orders(&candidate) {
                        tracing::warn!("Failed to copy bundled build orders from {:?}: {}", candidate, e);
                    }
                }
            }

            let settings = storage.load_settings().unwrap_or_default();
            let app_state = AppState {
                settings,
                ..AppState::default()
            };

            app.manage(Mutex::new(app_state));
            app.manage(Mutex::new(CaptureHandle { stop_tx: None }));
            app.manage(storage);

            hotkeys::setup_hotkeys(&app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::load_build_order_cmd,
            ipc::list_build_orders_cmd,
            ipc::advance_step,
            ipc::previous_step,
            ipc::reset_steps,
            ipc::get_settings,
            ipc::get_calibration,
            ipc::start_capture,
            ipc::stop_capture,
            ipc::generate_calibration_image,
            ipc::capture_calibration_screenshot,
            ipc::set_calibration,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
