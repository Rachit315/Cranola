use std::path::Path;

use serde::Serialize;

/// Basic facts about the running app, shown in the shell and in diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub os: String,
    pub arch: String,
    pub data_dir: String,
}

impl AppInfo {
    pub fn new(name: &str, version: &str, data_dir: &Path) -> Self {
        Self {
            name: name.to_owned(),
            version: version.to_owned(),
            os: std::env::consts::OS.to_owned(),
            arch: std::env::consts::ARCH.to_owned(),
            data_dir: data_dir.display().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_with_camel_case_keys() {
        let info = AppInfo::new("Scribewell", "1.2.3", Path::new("data"));
        let value = serde_json::to_value(&info).unwrap();
        assert_eq!(value["name"], "Scribewell");
        assert_eq!(value["version"], "1.2.3");
        assert_eq!(value["dataDir"], "data");
        assert_eq!(value["os"], std::env::consts::OS);
    }
}
