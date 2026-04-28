use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use tauri::{AppHandle, Manager};

use crate::ipc::emit_step_changed;
use crate::state::AppState;
use std::sync::Mutex;

struct HotkeyIds {
    advance: u32,
    previous: u32,
    reset: u32,
    toggle_visibility: u32,
}

pub fn setup_hotkeys(app: &AppHandle) {
    let manager = GlobalHotKeyManager::new().expect("Failed to create hotkey manager");

    let mods = Modifiers::CONTROL | Modifiers::ALT;

    let advance = HotKey::new(Some(mods), Code::ArrowRight);
    let previous = HotKey::new(Some(mods), Code::ArrowLeft);
    let reset = HotKey::new(Some(mods), Code::KeyR);
    let toggle_visibility = HotKey::new(Some(mods), Code::KeyH);

    manager.register(advance).expect("Failed to register advance hotkey");
    manager.register(previous).expect("Failed to register previous hotkey");
    manager.register(reset).expect("Failed to register reset hotkey");
    manager.register(toggle_visibility).expect("Failed to register toggle_visibility hotkey");

    let ids = HotkeyIds {
        advance: advance.id(),
        previous: previous.id(),
        reset: reset.id(),
        toggle_visibility: toggle_visibility.id(),
    };

    // Leak the manager so it stays alive for the lifetime of the application
    std::mem::forget(manager);

    let app_handle = app.clone();
    std::thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();
        loop {
            if let Ok(event) = receiver.recv() {
                let id = event.id();
                if id == ids.advance {
                    let state_mutex = app_handle.state::<Mutex<AppState>>();
                    let mut s = state_mutex.lock().unwrap();
                    if let Some(ref bo) = s.current_build_order {
                        if s.current_step_index < bo.steps.len() - 1 {
                            s.current_step_index += 1;
                        }
                    }
                    emit_step_changed(&app_handle, &s);
                } else if id == ids.previous {
                    let state_mutex = app_handle.state::<Mutex<AppState>>();
                    let mut s = state_mutex.lock().unwrap();
                    if s.current_step_index > 0 {
                        s.current_step_index -= 1;
                    }
                    emit_step_changed(&app_handle, &s);
                } else if id == ids.reset {
                    let state_mutex = app_handle.state::<Mutex<AppState>>();
                    let mut s = state_mutex.lock().unwrap();
                    s.current_step_index = 0;
                    emit_step_changed(&app_handle, &s);
                } else if id == ids.toggle_visibility {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        if let Ok(visible) = window.is_visible() {
                            if visible {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                            }
                        }
                    }
                }
            }
        }
    });
}
