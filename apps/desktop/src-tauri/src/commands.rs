//! IPC commands. Each command is a thin adapter over a service and maps
//! failures to `AppError`, which serializes to the shape the UI expects.

use sw_core::{AppError, AppInfo};
use tauri::{AppHandle, Manager, Runtime};

#[tauri::command]
pub fn get_app_info<R: Runtime>(app: AppHandle<R>) -> Result<AppInfo, AppError> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|err| AppError::internal(format!("cannot resolve app data dir: {err}")))?;
    std::fs::create_dir_all(&data_dir).map_err(|source| AppError::DataDir { source })?;

    let package = app.package_info();
    Ok(AppInfo::new(
        &package.name,
        &package.version.to_string(),
        &data_dir,
    ))
}
