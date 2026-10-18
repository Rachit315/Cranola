//! IPC commands. Each command is a thin adapter over a service and maps
//! failures to `AppError`, which serializes to the shape the UI expects.

use std::time::Duration;

use serde::Deserialize;
use sw_audio::{AudioDevice, LoopbackCheck, MicCheck, MicPermission};
use sw_core::settings::{OnboardingState, Settings, SettingsUpdate};
use sw_core::{AppError, AppInfo};
use sw_llm::ollama::OllamaStatus;
use sw_secrets::{SecretKey, SecretStatus};
use tauri::{AppHandle, Runtime, State};
use ts_rs::TS;

use crate::os::{self, OpenTarget};
use crate::state::{AppState, StartupStatus};

const MIC_CHECK_DURATION: Duration = Duration::from_millis(2500);
const LOOPBACK_CHECK_DURATION: Duration = Duration::from_millis(1500);

async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, AppError> + Send + 'static,
) -> Result<T, AppError> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| AppError::internal(format!("background task failed: {e}")))?
}

#[tauri::command]
pub fn get_startup_status(state: State<'_, AppState>) -> StartupStatus {
    state.startup_status()
}

#[tauri::command]
pub fn get_app_info<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<AppInfo, AppError> {
    let paths = state.paths()?;
    let package = app.package_info();
    Ok(AppInfo::new(
        &package.name,
        &package.version.to_string(),
        &paths.data_dir,
        &paths.log_dir,
    ))
}

#[tauri::command]
pub fn open_location(target: OpenTarget, state: State<'_, AppState>) -> Result<(), AppError> {
    os::open(target, state.paths()?)
}

#[derive(Debug, Clone, Copy, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum FrontendLogLevel {
    Error,
    Warn,
    Info,
}

/// Lets the UI record its own errors in the app log.
#[tauri::command]
pub fn log_frontend(level: FrontendLogLevel, message: String) {
    let message: String = message.chars().take(4000).collect();
    match level {
        FrontendLogLevel::Error => tracing::error!(target: "frontend", "{message}"),
        FrontendLogLevel::Warn => tracing::warn!(target: "frontend", "{message}"),
        FrontendLogLevel::Info => tracing::info!(target: "frontend", "{message}"),
    }
}

// Settings -------------------------------------------------------------------

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, AppError> {
    state
        .services()?
        .db
        .with_conn(|c| sw_db::settings::load_all(c))
}

#[tauri::command]
pub fn update_settings(
    update: SettingsUpdate,
    state: State<'_, AppState>,
) -> Result<Settings, AppError> {
    state
        .services()?
        .db
        .with_conn(|c| sw_db::settings::apply_update(c, &update))
}

#[tauri::command]
pub fn complete_onboarding(state: State<'_, AppState>) -> Result<Settings, AppError> {
    let update = SettingsUpdate::Onboarding(OnboardingState {
        completed_at: Some(sw_core::now_ms()),
    });
    state
        .services()?
        .db
        .with_conn(|c| sw_db::settings::apply_update(c, &update))
}

// Secrets --------------------------------------------------------------------

#[tauri::command]
pub fn list_secret_status(state: State<'_, AppState>) -> Result<Vec<SecretStatus>, AppError> {
    let store = state.services()?.secrets.as_ref();
    SecretKey::ALL
        .into_iter()
        .map(|key| sw_secrets::status(store, key))
        .collect()
}

/// Write-only: the value goes straight to the keychain and is never returned.
#[tauri::command]
pub fn set_secret(
    key: SecretKey,
    value: String,
    state: State<'_, AppState>,
) -> Result<SecretStatus, AppError> {
    let store = state.services()?.secrets.as_ref();
    store.set(key, &value)?;
    tracing::info!(key = key.account(), "secret stored");
    sw_secrets::status(store, key)
}

#[tauri::command]
pub fn delete_secret(key: SecretKey, state: State<'_, AppState>) -> Result<SecretStatus, AppError> {
    let store = state.services()?.secrets.as_ref();
    store.delete(key)?;
    tracing::info!(key = key.account(), "secret deleted");
    sw_secrets::status(store, key)
}

// Audio ----------------------------------------------------------------------

#[tauri::command]
pub async fn list_audio_devices() -> Result<Vec<AudioDevice>, AppError> {
    blocking(sw_audio::list_devices).await
}

#[tauri::command]
pub fn get_microphone_permission() -> MicPermission {
    sw_audio::microphone_permission()
}

#[tauri::command]
pub async fn check_microphone(device_id: Option<String>) -> Result<MicCheck, AppError> {
    blocking(move || sw_audio::check_microphone(device_id.as_deref(), MIC_CHECK_DURATION)).await
}

#[tauri::command]
pub async fn check_system_audio(device_id: Option<String>) -> Result<LoopbackCheck, AppError> {
    blocking(move || sw_audio::check_loopback(device_id.as_deref(), LOOPBACK_CHECK_DURATION)).await
}

// AI providers ---------------------------------------------------------------

#[tauri::command]
pub async fn detect_ollama(
    base_url: Option<String>,
    state: State<'_, AppState>,
) -> Result<OllamaStatus, AppError> {
    let url = match base_url {
        Some(url) => url,
        None => {
            state
                .services()?
                .db
                .with_conn(|c| sw_db::settings::load_all(c))?
                .ai
                .ollama_base_url
        }
    };
    sw_llm::ollama::detect(&url).await
}
