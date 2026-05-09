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
    setup_panic_hook();
    setup_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            let storage = Storage::new(app_data_dir)?;

            let bundled_dir = app
                .path()
                .resource_dir()
                .expect("Failed to get resource dir")
                .join("build-orders");
            let _ = storage.copy_bundled_build_orders(&bundled_dir);

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
