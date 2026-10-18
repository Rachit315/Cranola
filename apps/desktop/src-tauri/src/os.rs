//! Opening folders and system settings pages with the OS shell.

use std::process::Command;

use serde::Deserialize;
use sw_core::AppError;
use ts_rs::TS;

use crate::state::AppPaths;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum OpenTarget {
    DataFolder,
    LogsFolder,
    MicrophoneSettings,
}

fn microphone_settings_uri() -> Option<&'static str> {
    if cfg!(windows) {
        Some("ms-settings:privacy-microphone")
    } else if cfg!(target_os = "macos") {
        Some("x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone")
    } else {
        None
    }
}

fn shell_open(target: &str) -> Result<(), AppError> {
    let mut cmd = if cfg!(windows) {
        Command::new("explorer")
    } else if cfg!(target_os = "macos") {
        Command::new("open")
    } else {
        Command::new("xdg-open")
    };
    // explorer.exe returns exit code 1 even on success, so only a failure to
    // launch the process is treated as an error.
    cmd.arg(target)
        .spawn()
        .map(|_| ())
        .map_err(|e| AppError::internal(format!("could not open {target}: {e}")))
}

pub fn open(target: OpenTarget, paths: &AppPaths) -> Result<(), AppError> {
    match target {
        OpenTarget::DataFolder => shell_open(&paths.data_dir.display().to_string()),
        OpenTarget::LogsFolder => shell_open(&paths.log_dir.display().to_string()),
        OpenTarget::MicrophoneSettings => match microphone_settings_uri() {
            Some(uri) => shell_open(uri),
            None => Err(AppError::invalid(
                "target",
                "this system has no microphone privacy page; check your desktop's sound settings",
            )),
        },
    }
}
