// Hide the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod logging;
mod os;
mod state;

use std::process::ExitCode;
use std::sync::Arc;

use sw_core::AppError;
use sw_secrets::KeychainStore;
use tauri::Manager;

use crate::state::{AppPaths, AppState, KEYCHAIN_SERVICE, StartupError};

/// Builds the app state. Never fails: problems are recorded in the state and
/// shown on the startup error screen instead of crashing the app.
fn build_state(app: &tauri::App) -> AppState {
    let resolver = app.path();
    let paths = match (resolver.app_data_dir(), resolver.app_log_dir()) {
        (Ok(data_dir), Ok(log_dir)) => AppPaths { data_dir, log_dir },
        (Err(e), _) | (_, Err(e)) => {
            let err = AppError::internal(format!("cannot resolve app folders: {e}"));
            return AppState::new(None, Err(StartupError::from(&err)), None);
        }
    };

    let log_guard = match logging::init(&paths.log_dir) {
        Ok(guard) => Some(guard),
        Err(err) => {
            eprintln!("Scribewell: file logging disabled: {err}");
            None
        }
    };
    tracing::info!(
        version = %app.package_info().version,
        data_dir = %paths.data_dir.display(),
        "starting"
    );

    let services = state::ensure_dir(&paths.data_dir).and_then(|()| {
        state::open_services(&paths, Arc::new(KeychainStore::new(KEYCHAIN_SERVICE)))
    });
    let services = services.map_err(|err| {
        tracing::error!(code = err.code(), detail = ?err.detail(), "startup failed");
        StartupError::from(&err)
    });
    AppState::new(Some(paths), services, log_guard)
}

fn main() -> ExitCode {
    let result = tauri::Builder::default()
        .setup(|app| {
            let state = build_state(app);
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_startup_status,
            commands::get_app_info,
            commands::open_location,
            commands::log_frontend,
            commands::get_settings,
            commands::update_settings,
            commands::complete_onboarding,
            commands::list_secret_status,
            commands::set_secret,
            commands::delete_secret,
            commands::list_audio_devices,
            commands::get_microphone_permission,
            commands::check_microphone,
            commands::check_system_audio,
            commands::detect_ollama,
        ])
        .run(tauri::generate_context!());

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            tracing::error!(%err, "tauri runtime error");
            eprintln!("Scribewell failed to start: {err}");
            ExitCode::FAILURE
        }
    }
}
