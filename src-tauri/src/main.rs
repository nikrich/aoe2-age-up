#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod build_order;
mod capture;
mod error;
mod ocr;
mod state;
mod storage;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
