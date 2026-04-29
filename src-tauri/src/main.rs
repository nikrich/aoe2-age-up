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

fn main() {
    tracing_subscriber::fmt::init();

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
