use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::build_order::parser::load_build_order;
use crate::build_order::{BuildOrder, BuildOrderMeta, Step};
use crate::error::AppError;
use crate::state::{AppState, Calibration, Settings};
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
