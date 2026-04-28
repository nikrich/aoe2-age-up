#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod build_order;
mod error;
mod state;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
