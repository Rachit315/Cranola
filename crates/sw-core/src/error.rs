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

    #[error("Unexpected internal error")]
    Internal { detail: String },
}

impl AppError {
    pub fn internal(detail: impl Into<String>) -> Self {
        Self::Internal {
            detail: detail.into(),
        }
    }

    /// Stable identifier, namespaced by subsystem.
    pub fn code(&self) -> &'static str {
        match self {
            Self::DataDir { .. } => "io.data_dir",
            Self::Internal { .. } => "internal",
        }
    }

    /// Whether retrying the same operation unchanged may succeed.
    pub fn retryable(&self) -> bool {
        match self {
            Self::DataDir { .. } | Self::Internal { .. } => false,
        }
    }

    /// Technical detail for diagnostics. Never contains secrets.
    pub fn detail(&self) -> Option<String> {
        match self {
            Self::DataDir { source } => Some(source.to_string()),
            Self::Internal { detail } => Some(detail.clone()),
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("AppError", 4)?;
        s.serialize_field("code", self.code())?;
        s.serialize_field("message", &self.to_string())?;
        s.serialize_field("detail", &self.detail())?;
        s.serialize_field("retryable", &self.retryable())?;
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
            })
        );
    }

    #[test]
    fn internal_errors_keep_detail() {
        let value = serde_json::to_value(AppError::internal("path resolver failed")).unwrap();
        assert_eq!(value["code"], "internal");
        assert_eq!(value["detail"], "path resolver failed");
    }
}
