//! User settings. Each section is persisted as one JSON row keyed by
//! [`SettingsSection::KEY`], so sections can evolve independently and missing
//! fields fall back to defaults. Secrets never live here; see `sw-secrets`.

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use ts_rs::TS;

use crate::AppError;

pub trait SettingsSection: Serialize + DeserializeOwned + Default {
    const KEY: &'static str;

    /// Reject values that would break the app if persisted.
    fn validate(&self) -> Result<(), AppError> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct GeneralSettings {
    pub theme: Theme,
}

impl SettingsSection for GeneralSettings {
    const KEY: &'static str = "general";
}

/// Context about the user that steers note generation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct ProfileSettings {
    pub name: String,
    pub role: String,
    pub company: String,
    /// Free-form context, e.g. "I run customer discovery calls for a B2B analytics product."
    pub about: String,
}

pub const PROFILE_SHORT_FIELD_MAX: usize = 120;
pub const PROFILE_ABOUT_MAX: usize = 2000;

impl SettingsSection for ProfileSettings {
    const KEY: &'static str = "profile";

    fn validate(&self) -> Result<(), AppError> {
        for (field, value) in [
            ("name", &self.name),
            ("role", &self.role),
            ("company", &self.company),
        ] {
            if value.chars().count() > PROFILE_SHORT_FIELD_MAX {
                return Err(AppError::invalid(
                    field,
                    format!("must be at most {PROFILE_SHORT_FIELD_MAX} characters"),
                ));
            }
        }
        if self.about.chars().count() > PROFILE_ABOUT_MAX {
            return Err(AppError::invalid(
                "about",
                format!("must be at most {PROFILE_ABOUT_MAX} characters"),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct AudioSettings {
    /// Microphone device id; `None` follows the system default.
    pub input_device_id: Option<String>,
    /// Output device whose mix is captured as "them"; `None` follows the system default.
    pub output_device_id: Option<String>,
}

impl SettingsSection for AudioSettings {
    const KEY: &'static str = "audio";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum LlmProvider {
    Ollama,
    Anthropic,
    Openai,
    Gemini,
    Openrouter,
}

pub const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct AiSettings {
    /// Provider used for enhancing notes and chat; `None` until the user picks one.
    pub llm_provider: Option<LlmProvider>,
    pub ollama_base_url: String,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            llm_provider: None,
            ollama_base_url: DEFAULT_OLLAMA_URL.to_owned(),
        }
    }
}

impl SettingsSection for AiSettings {
    const KEY: &'static str = "ai";

    fn validate(&self) -> Result<(), AppError> {
        let url = self.ollama_base_url.trim();
        let rest = url
            .strip_prefix("http://")
            .or_else(|| url.strip_prefix("https://"))
            .ok_or_else(|| {
                AppError::invalid("ollamaBaseUrl", "must start with http:// or https://")
            })?;
        if rest.is_empty() || rest.starts_with('/') || rest.contains(char::is_whitespace) {
            return Err(AppError::invalid("ollamaBaseUrl", "must include a host"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct OnboardingState {
    /// Unix ms when the user finished (or skipped) onboarding.
    pub completed_at: Option<i64>,
}

impl SettingsSection for OnboardingState {
    const KEY: &'static str = "onboarding";
}

/// All settings, as sent to the UI.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Settings {
    pub general: GeneralSettings,
    pub profile: ProfileSettings,
    pub audio: AudioSettings,
    pub ai: AiSettings,
    pub onboarding: OnboardingState,
}

/// One section replaced as a whole, as sent from the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "section", content = "value", rename_all = "camelCase")]
#[ts(export)]
pub enum SettingsUpdate {
    General(GeneralSettings),
    Profile(ProfileSettings),
    Audio(AudioSettings),
    Ai(AiSettings),
    Onboarding(OnboardingState),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let ai: AiSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(ai, AiSettings::default());
        let ai: AiSettings = serde_json::from_str(r#"{"llmProvider":"anthropic"}"#).unwrap();
        assert_eq!(ai.llm_provider, Some(LlmProvider::Anthropic));
        assert_eq!(ai.ollama_base_url, DEFAULT_OLLAMA_URL);
    }

    #[test]
    fn ollama_url_validation() {
        let mut ai = AiSettings::default();
        assert!(ai.validate().is_ok());
        ai.ollama_base_url = "https://ollama.local:8443".into();
        assert!(ai.validate().is_ok());
        for bad in [
            "localhost:11434",
            "ftp://x",
            "http://",
            "http:///x",
            "http://a b",
        ] {
            ai.ollama_base_url = bad.into();
            assert!(ai.validate().is_err(), "{bad} should be rejected");
        }
    }

    #[test]
    fn profile_length_limits() {
        let mut p = ProfileSettings {
            name: "Ada".into(),
            ..Default::default()
        };
        assert!(p.validate().is_ok());
        p.about = "x".repeat(PROFILE_ABOUT_MAX + 1);
        let err = p.validate().unwrap_err();
        assert_eq!(err.code(), "input.invalid");
    }

    #[test]
    fn update_is_tagged_by_section() {
        let json = r#"{"section":"general","value":{"theme":"dark"}}"#;
        let update: SettingsUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(
            update,
            SettingsUpdate::General(GeneralSettings { theme: Theme::Dark })
        );
    }
}
