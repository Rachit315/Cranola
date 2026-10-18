//! Settings persistence: one JSON row per section.

use rusqlite::{Connection, OptionalExtension};
use sw_core::AppError;
use sw_core::settings::{
    AiSettings, AudioSettings, GeneralSettings, OnboardingState, ProfileSettings, Settings,
    SettingsSection, SettingsUpdate,
};

use crate::db_err;

/// Reads a section. Missing rows yield defaults. A row that no longer parses
/// (e.g. written by a buggy build) is logged and replaced by defaults rather
/// than making the app unusable.
pub fn get_section<T: SettingsSection>(conn: &Connection) -> Result<T, AppError> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = ?1",
            [T::KEY],
            |r| r.get(0),
        )
        .optional()
        .map_err(db_err)?;
    match raw {
        None => Ok(T::default()),
        Some(json) => match serde_json::from_str(&json) {
            Ok(value) => Ok(value),
            Err(err) => {
                tracing::warn!(key = T::KEY, %err, "unreadable settings section; using defaults");
                Ok(T::default())
            }
        },
    }
}

pub fn put_section<T: SettingsSection>(conn: &Connection, value: &T) -> Result<(), AppError> {
    value.validate()?;
    let json = serde_json::to_string(value)
        .map_err(|e| AppError::internal(format!("serialize settings: {e}")))?;
    conn.execute(
        "INSERT INTO settings (key, value_json, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json,
                                        updated_at = excluded.updated_at",
        rusqlite::params![T::KEY, json, sw_core::now_ms()],
    )
    .map_err(db_err)?;
    Ok(())
}

pub fn load_all(conn: &Connection) -> Result<Settings, AppError> {
    Ok(Settings {
        general: get_section::<GeneralSettings>(conn)?,
        profile: get_section::<ProfileSettings>(conn)?,
        audio: get_section::<AudioSettings>(conn)?,
        ai: get_section::<AiSettings>(conn)?,
        onboarding: get_section::<OnboardingState>(conn)?,
    })
}

pub fn apply_update(conn: &Connection, update: &SettingsUpdate) -> Result<Settings, AppError> {
    match update {
        SettingsUpdate::General(v) => put_section(conn, v)?,
        SettingsUpdate::Profile(v) => put_section(conn, v)?,
        SettingsUpdate::Audio(v) => put_section(conn, v)?,
        SettingsUpdate::Ai(v) => put_section(conn, v)?,
        SettingsUpdate::Onboarding(v) => put_section(conn, v)?,
    }
    load_all(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Db;
    use sw_core::settings::{LlmProvider, Theme};

    #[test]
    fn defaults_when_empty() {
        let db = Db::open_in_memory().unwrap();
        let s = db.with_conn(|c| load_all(c)).unwrap();
        assert_eq!(s, Settings::default());
    }

    #[test]
    fn update_round_trips() {
        let db = Db::open_in_memory().unwrap();
        let s = db
            .with_conn(|c| {
                apply_update(
                    c,
                    &SettingsUpdate::Ai(AiSettings {
                        llm_provider: Some(LlmProvider::Ollama),
                        ..Default::default()
                    }),
                )?;
                apply_update(
                    c,
                    &SettingsUpdate::General(GeneralSettings { theme: Theme::Dark }),
                )
            })
            .unwrap();
        assert_eq!(s.general.theme, Theme::Dark);
        assert_eq!(s.ai.llm_provider, Some(LlmProvider::Ollama));
        assert_eq!(s.profile, ProfileSettings::default());
    }

    #[test]
    fn invalid_values_are_not_persisted() {
        let db = Db::open_in_memory().unwrap();
        let err = db
            .with_conn(|c| {
                apply_update(
                    c,
                    &SettingsUpdate::Ai(AiSettings {
                        llm_provider: None,
                        ollama_base_url: "not a url".into(),
                    }),
                )
            })
            .unwrap_err();
        assert_eq!(err.code(), "input.invalid");
        let ai = db.with_conn(|c| get_section::<AiSettings>(c)).unwrap();
        assert_eq!(ai, AiSettings::default());
    }

    #[test]
    fn unreadable_section_falls_back_to_defaults() {
        let db = Db::open_in_memory().unwrap();
        let theme = db
            .with_conn(|c| {
                c.execute(
                    "INSERT INTO settings (key, value_json, updated_at) VALUES ('general', '{\"theme\": 42}', 0)",
                    [],
                )
                .map_err(db_err)?;
                get_section::<GeneralSettings>(c)
            })
            .unwrap();
        assert_eq!(theme, GeneralSettings::default());
    }
}
