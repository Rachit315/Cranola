//! Credential storage in the OS keychain (Windows Credential Manager, macOS
//! Keychain, Secret Service on Linux). There is deliberately no plain-text
//! fallback: if the keychain is unavailable, storing a secret fails.
//!
//! Secrets never travel to the UI. The UI only learns whether a secret is
//! configured and its last four characters, via [`SecretStatus`].

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use sw_core::AppError;
use ts_rs::TS;

/// Every credential the app can store. Adding a variant is the only way to
/// introduce a new secret, which keeps the set auditable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SecretKey {
    AnthropicApiKey,
    OpenaiApiKey,
    GeminiApiKey,
    OpenrouterApiKey,
    DeepgramApiKey,
    GroqApiKey,
}

impl SecretKey {
    pub const ALL: [SecretKey; 6] = [
        Self::AnthropicApiKey,
        Self::OpenaiApiKey,
        Self::GeminiApiKey,
        Self::OpenrouterApiKey,
        Self::DeepgramApiKey,
        Self::GroqApiKey,
    ];

    /// Account name under which the secret is stored in the keychain.
    pub fn account(self) -> &'static str {
        match self {
            Self::AnthropicApiKey => "anthropic-api-key",
            Self::OpenaiApiKey => "openai-api-key",
            Self::GeminiApiKey => "gemini-api-key",
            Self::OpenrouterApiKey => "openrouter-api-key",
            Self::DeepgramApiKey => "deepgram-api-key",
            Self::GroqApiKey => "groq-api-key",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SecretStatus {
    pub key: SecretKey,
    pub configured: bool,
    /// Last four characters, so users can tell which key is stored.
    pub hint: Option<String>,
}

pub trait SecretStore: Send + Sync + Debug {
    fn set(&self, key: SecretKey, value: &str) -> Result<(), AppError>;
    fn get(&self, key: SecretKey) -> Result<Option<String>, AppError>;
    fn delete(&self, key: SecretKey) -> Result<(), AppError>;
}

pub const MAX_SECRET_LEN: usize = 4096;

/// Trims and validates a secret before storage.
pub fn normalize(value: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::invalid("secret", "must not be empty"));
    }
    if trimmed.len() > MAX_SECRET_LEN {
        return Err(AppError::invalid(
            "secret",
            format!("must be at most {MAX_SECRET_LEN} characters"),
        ));
    }
    if trimmed.chars().any(char::is_whitespace) {
        return Err(AppError::invalid(
            "secret",
            "must not contain spaces or line breaks",
        ));
    }
    Ok(trimmed.to_owned())
}

pub fn status(store: &dyn SecretStore, key: SecretKey) -> Result<SecretStatus, AppError> {
    let value = store.get(key)?;
    Ok(SecretStatus {
        key,
        configured: value.is_some(),
        hint: value.map(|v| {
            let chars: Vec<char> = v.chars().collect();
            chars[chars.len().saturating_sub(4)..].iter().collect()
        }),
    })
}

/// The real OS keychain.
#[derive(Debug)]
pub struct KeychainStore {
    service: String,
}

impl KeychainStore {
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    /// Whether the platform keychain could be initialized.
    pub fn availability() -> Result<(), AppError> {
        keyring::Entry::store_status()
            .as_ref()
            .map(|_| ())
            .map_err(|err| AppError::SecretStoreUnavailable {
                detail: err.to_string(),
            })
    }

    fn entry(&self, key: SecretKey) -> Result<keyring::Entry, AppError> {
        keyring::Entry::new(&self.service, key.account()).map_err(map_err)
    }
}

fn map_err(err: keyring::Error) -> AppError {
    match err {
        keyring::Error::NoDefaultStore | keyring::Error::NoStorageAccess(_) => {
            AppError::SecretStoreUnavailable {
                detail: err.to_string(),
            }
        }
        other => AppError::SecretStore {
            detail: other.to_string(),
        },
    }
}

impl SecretStore for KeychainStore {
    fn set(&self, key: SecretKey, value: &str) -> Result<(), AppError> {
        let value = normalize(value)?;
        self.entry(key)?.set_password(&value).map_err(map_err)
    }

    fn get(&self, key: SecretKey) -> Result<Option<String>, AppError> {
        match self.entry(key)?.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(err) => Err(map_err(err)),
        }
    }

    fn delete(&self, key: SecretKey) -> Result<(), AppError> {
        match self.entry(key)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(map_err(err)),
        }
    }
}

/// In-process store used by tests of crates that depend on secrets.
#[derive(Debug, Default)]
pub struct MemoryStore {
    values: Mutex<HashMap<SecretKey, String>>,
}

impl MemoryStore {
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, HashMap<SecretKey, String>>, AppError> {
        self.values
            .lock()
            .map_err(|_| AppError::internal("secret store lock poisoned"))
    }
}

impl SecretStore for MemoryStore {
    fn set(&self, key: SecretKey, value: &str) -> Result<(), AppError> {
        let value = normalize(value)?;
        self.lock()?.insert(key, value);
        Ok(())
    }

    fn get(&self, key: SecretKey) -> Result<Option<String>, AppError> {
        Ok(self.lock()?.get(&key).cloned())
    }

    fn delete(&self, key: SecretKey) -> Result<(), AppError> {
        self.lock()?.remove(&key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_trims_and_validates() {
        assert_eq!(normalize("  sk-abc123 \n").unwrap(), "sk-abc123");
        assert_eq!(normalize("   ").unwrap_err().code(), "input.invalid");
        assert_eq!(normalize("sk abc").unwrap_err().code(), "input.invalid");
        assert!(normalize(&"x".repeat(MAX_SECRET_LEN + 1)).is_err());
    }

    #[test]
    fn status_reports_hint_without_value() {
        let store = MemoryStore::default();
        let empty = status(&store, SecretKey::GroqApiKey).unwrap();
        assert!(!empty.configured);
        assert_eq!(empty.hint, None);

        store
            .set(SecretKey::GroqApiKey, "gsk_0123456789abcd")
            .unwrap();
        let st = status(&store, SecretKey::GroqApiKey).unwrap();
        assert!(st.configured);
        assert_eq!(st.hint.as_deref(), Some("abcd"));
        let json = serde_json::to_string(&st).unwrap();
        assert!(!json.contains("gsk_"), "full secret must not be serialized");
    }

    #[test]
    fn accounts_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for key in SecretKey::ALL {
            assert!(seen.insert(key.account()));
        }
    }

    /// Exercises the real OS keychain with a throwaway service name. Runs on
    /// every developer machine and CI runner that has a keychain; on a
    /// headless Linux runner without Secret Service it reports why it could
    /// not run instead of passing silently.
    #[test]
    fn real_keychain_round_trip() {
        if let Err(err) = KeychainStore::availability() {
            eprintln!("keychain unavailable on this machine, round trip not exercised: {err:?}");
            return;
        }
        let service = format!("dev.scribewell.test.{}", std::process::id());
        let store = KeychainStore::new(service);
        let key = SecretKey::DeepgramApiKey;

        store.delete(key).unwrap();
        assert_eq!(store.get(key).unwrap(), None);
        store.set(key, "test-secret-value-123").unwrap();
        assert_eq!(
            store.get(key).unwrap().as_deref(),
            Some("test-secret-value-123")
        );
        store.set(key, "replaced-value-456").unwrap();
        assert_eq!(
            store.get(key).unwrap().as_deref(),
            Some("replaced-value-456")
        );
        store.delete(key).unwrap();
        assert_eq!(store.get(key).unwrap(), None);
        store.delete(key).unwrap();
    }
}
