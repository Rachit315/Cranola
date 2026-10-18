//! Application state assembled at startup.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Serialize;
use sw_core::AppError;
use sw_db::Db;
use sw_secrets::SecretStore;
use tracing_appender::non_blocking::WorkerGuard;
use ts_rs::TS;

/// Keychain service name under which all credentials are stored.
pub const KEYCHAIN_SERVICE: &str = "dev.scribewell.desktop";

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub log_dir: PathBuf,
}

impl AppPaths {
    pub fn db_file(&self) -> PathBuf {
        self.data_dir.join("scribewell.db")
    }

    pub fn backup_dir(&self) -> PathBuf {
        self.data_dir.join("backups")
    }
}

#[derive(Debug)]
pub struct Services {
    pub db: Db,
    pub secrets: Arc<dyn SecretStore>,
}

/// Why the app could not start, shown on a dedicated error screen.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StartupError {
    pub code: String,
    pub message: String,
    pub detail: Option<String>,
}

impl From<&AppError> for StartupError {
    fn from(err: &AppError) -> Self {
        Self {
            code: err.code().to_owned(),
            message: err.to_string(),
            detail: err.detail(),
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StartupStatus {
    pub ready: bool,
    pub error: Option<StartupError>,
    pub log_dir: Option<String>,
}

#[derive(Debug)]
pub struct AppState {
    pub paths: Option<AppPaths>,
    services: Result<Services, StartupError>,
    _log_guard: Option<WorkerGuard>,
}

impl AppState {
    pub fn new(
        paths: Option<AppPaths>,
        services: Result<Services, StartupError>,
        log_guard: Option<WorkerGuard>,
    ) -> Self {
        Self {
            paths,
            services,
            _log_guard: log_guard,
        }
    }

    pub fn services(&self) -> Result<&Services, AppError> {
        self.services.as_ref().map_err(|e| {
            AppError::internal(format!("the app did not start correctly: {}", e.message))
        })
    }

    pub fn paths(&self) -> Result<&AppPaths, AppError> {
        self.paths
            .as_ref()
            .ok_or_else(|| AppError::internal("app folders could not be resolved"))
    }

    pub fn startup_status(&self) -> StartupStatus {
        StartupStatus {
            ready: self.services.is_ok(),
            error: self.services.as_ref().err().cloned(),
            log_dir: self.paths.as_ref().map(|p| p.log_dir.display().to_string()),
        }
    }
}

/// Opens the database and keychain. Pure so it can be tested without Tauri.
pub fn open_services(
    paths: &AppPaths,
    secrets: Arc<dyn SecretStore>,
) -> Result<Services, AppError> {
    std::fs::create_dir_all(&paths.data_dir).map_err(|source| AppError::DataDir { source })?;
    let (db, report) = Db::open(&paths.db_file(), &paths.backup_dir())?;
    if report.from != report.to {
        tracing::info!(from = report.from, to = report.to, backup = ?report.backup, "database migrated");
    }
    Ok(Services { db, secrets })
}

pub fn ensure_dir(dir: &Path) -> Result<(), AppError> {
    std::fs::create_dir_all(dir).map_err(|source| AppError::DataDir { source })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sw_secrets::MemoryStore;

    fn paths(root: &Path) -> AppPaths {
        AppPaths {
            data_dir: root.join("data"),
            log_dir: root.join("logs"),
        }
    }

    #[test]
    fn opens_services_in_fresh_folder() {
        let dir = tempfile::tempdir().unwrap();
        let p = paths(dir.path());
        let services = open_services(&p, Arc::new(MemoryStore::default())).unwrap();
        assert!(p.db_file().exists());
        let state = AppState::new(Some(p), Ok(services), None);
        let status = state.startup_status();
        assert!(status.ready);
        assert!(status.error.is_none());
    }

    #[test]
    fn startup_failure_is_reported_not_panicked() {
        let dir = tempfile::tempdir().unwrap();
        let p = paths(dir.path());
        // A file where the data folder should be makes the folder uncreatable.
        std::fs::write(&p.data_dir, b"not a folder").unwrap();
        let err = open_services(&p, Arc::new(MemoryStore::default())).unwrap_err();
        assert_eq!(err.code(), "io.data_dir");

        let state = AppState::new(Some(p), Err(StartupError::from(&err)), None);
        let status = state.startup_status();
        assert!(!status.ready);
        assert_eq!(status.error.unwrap().code, "io.data_dir");
        assert!(state.services().is_err());
    }
}
