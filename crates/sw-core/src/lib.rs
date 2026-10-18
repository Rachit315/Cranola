//! Shared domain types for Scribewell. This crate must not depend on Tauri.

pub mod error;
pub mod info;
pub mod redact;
pub mod settings;

pub use error::AppError;
pub use info::AppInfo;

/// Current wall-clock time as Unix milliseconds.
pub fn now_ms() -> i64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    i64::try_from(now.as_millis()).unwrap_or(i64::MAX)
}
