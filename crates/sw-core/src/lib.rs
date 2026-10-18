//! Shared domain types for Scribewell. This crate must not depend on Tauri.

pub mod error;
pub mod info;

pub use error::AppError;
pub use info::AppInfo;
