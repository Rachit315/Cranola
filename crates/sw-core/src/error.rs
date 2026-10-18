//! The single error type that crosses the IPC boundary.
//!
//! Every variant has a stable machine-readable `code` that the UI maps to copy
//! and recovery actions, so error handling never depends on message text.

use serde::ser::{Serialize, SerializeStruct, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("The app data folder could not be prepared")]
    DataDir {
        #[source]
        source: std::io::Error,
    },

    #[error("The local database reported an error")]
    Database { detail: String },

    #[error("Upgrading the local database failed; your previous data was restored from backup")]
    MigrationFailed {
        version: u32,
        detail: String,
        backup: Option<String>,
    },

    #[error("The system keychain is not available")]
    SecretStoreUnavailable { detail: String },

    #[error("The system keychain reported an error")]
    SecretStore { detail: String },

    #[error("Invalid {field}: {reason}")]
    InvalidInput { field: String, reason: String },

    #[error("Microphone access is blocked by system privacy settings")]
    MicrophonePermissionDenied,

    #[error("No {kind} audio device was found")]
    NoAudioDevice { kind: &'static str },

    #[error("The audio device could not be used")]
    Audio { detail: String },

    #[error("Could not reach {service}")]
    Network { service: String, detail: String },

    #[error("Unexpected internal error")]
    Internal { detail: String },
}

impl AppError {
    pub fn internal(detail: impl Into<String>) -> Self {
        Self::Internal {
            detail: detail.into(),
        }
    }

    pub fn invalid(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidInput {
            field: field.into(),
            reason: reason.into(),
        }
    }

    /// Stable identifier, namespaced by subsystem.
    pub fn code(&self) -> &'static str {
        match self {
            Self::DataDir { .. } => "io.data_dir",
            Self::Database { .. } => "db.error",
            Self::MigrationFailed { .. } => "db.migration_failed",
            Self::SecretStoreUnavailable { .. } => "secrets.unavailable",
            Self::SecretStore { .. } => "secrets.error",
            Self::InvalidInput { .. } => "input.invalid",
            Self::MicrophonePermissionDenied => "audio.permission_denied",
            Self::NoAudioDevice { .. } => "audio.no_device",
            Self::Audio { .. } => "audio.error",
            Self::Network { .. } => "net.unreachable",
            Self::Internal { .. } => "internal",
        }
    }

    /// Whether retrying the same operation unchanged may succeed.
    pub fn retryable(&self) -> bool {
        matches!(
            self,
            Self::Network { .. } | Self::Audio { .. } | Self::NoAudioDevice { .. }
        )
    }

    /// A recovery action the UI can offer, e.g. opening a system settings page.
    pub fn action(&self) -> Option<&'static str> {
        match self {
            Self::MicrophonePermissionDenied => Some("open_settings:microphone"),
            Self::MigrationFailed { .. } | Self::Database { .. } => Some("open_logs"),
            _ => None,
        }
    }

    /// Technical detail for diagnostics. Never contains secrets.
    pub fn detail(&self) -> Option<String> {
        match self {
            Self::DataDir { source } => Some(source.to_string()),
            Self::Database { detail }
            | Self::SecretStoreUnavailable { detail }
            | Self::SecretStore { detail }
            | Self::Audio { detail }
            | Self::Network { detail, .. }
            | Self::Internal { detail } => Some(detail.clone()),
            Self::MigrationFailed {
                version,
                detail,
                backup,
            } => Some(match backup {
                Some(path) => format!("migration {version}: {detail} (backup: {path})"),
                None => format!("migration {version}: {detail}"),
            }),
            Self::InvalidInput { .. }
            | Self::MicrophonePermissionDenied
            | Self::NoAudioDevice { .. } => None,
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("AppError", 5)?;
        s.serialize_field("code", self.code())?;
        s.serialize_field("message", &self.to_string())?;
        s.serialize_field("detail", &self.detail())?;
        s.serialize_field("retryable", &self.retryable())?;
        s.serialize_field("action", &self.action())?;
        s.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_to_ipc_shape() {
        let err = AppError::DataDir {
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied"),
        };
        let value = serde_json::to_value(&err).unwrap();
        assert_eq!(
            value,
            json!({
                "code": "io.data_dir",
                "message": "The app data folder could not be prepared",
                "detail": "access denied",
                "retryable": false,
                "action": null,
            })
        );
    }

    #[test]
    fn internal_errors_keep_detail() {
        let value = serde_json::to_value(AppError::internal("path resolver failed")).unwrap();
        assert_eq!(value["code"], "internal");
        assert_eq!(value["detail"], "path resolver failed");
    }

    #[test]
    fn permission_denied_offers_settings_action() {
        let value = serde_json::to_value(AppError::MicrophonePermissionDenied).unwrap();
        assert_eq!(value["code"], "audio.permission_denied");
        assert_eq!(value["action"], "open_settings:microphone");
        assert_eq!(value["detail"], serde_json::Value::Null);
    }

    #[test]
    fn migration_failure_mentions_backup() {
        let err = AppError::MigrationFailed {
            version: 3,
            detail: "no such table".into(),
            backup: Some("C:/b/pre-v3.db".into()),
        };
        assert_eq!(
            err.detail().unwrap(),
            "migration 3: no such table (backup: C:/b/pre-v3.db)"
        );
        assert!(!err.retryable());
    }
}
