//! Microphone privacy state.
//!
//! On Windows, a desktop app whose microphone access is turned off in
//! Settings > Privacy & security > Microphone does not get an error when
//! opening the device: it silently receives zeros. So the consent store in
//! the registry is the reliable signal.

use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum MicPermission {
    Allowed,
    /// Microphone access is off for the whole device.
    DeniedSystemWide,
    /// Microphone access is on, but desktop apps are not allowed to use it.
    DeniedForDesktopApps,
    /// The platform has no queryable setting (or it could not be read); the
    /// real check happens when the device is opened.
    Unknown,
}

impl MicPermission {
    pub fn is_denied(self) -> bool {
        matches!(self, Self::DeniedSystemWide | Self::DeniedForDesktopApps)
    }
}

/// Interprets the consent-store values. `None` means the value is absent,
/// which Windows treats as allowed.
pub fn interpret_consent(device_wide: Option<&str>, desktop_apps: Option<&str>) -> MicPermission {
    let denied = |v: Option<&str>| v.is_some_and(|s| s.eq_ignore_ascii_case("deny"));
    if denied(device_wide) {
        MicPermission::DeniedSystemWide
    } else if denied(desktop_apps) {
        MicPermission::DeniedForDesktopApps
    } else {
        MicPermission::Allowed
    }
}

#[cfg(windows)]
pub fn microphone_permission() -> MicPermission {
    const BASE: &str = r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\microphone";
    let read = |root: &windows_registry::Key, path: &str| -> Option<String> {
        root.open(path).ok()?.get_string("Value").ok()
    };
    // A machine-wide policy (HKLM) overrides the user's choice.
    let machine = read(windows_registry::LOCAL_MACHINE, BASE);
    let user = read(windows_registry::CURRENT_USER, BASE);
    let desktop = read(
        windows_registry::CURRENT_USER,
        &format!(r"{BASE}\NonPackaged"),
    );
    match interpret_consent(machine.as_deref(), None) {
        MicPermission::DeniedSystemWide => MicPermission::DeniedSystemWide,
        _ => interpret_consent(user.as_deref(), desktop.as_deref()),
    }
}

#[cfg(not(windows))]
pub fn microphone_permission() -> MicPermission {
    MicPermission::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consent_interpretation() {
        assert_eq!(interpret_consent(None, None), MicPermission::Allowed);
        assert_eq!(
            interpret_consent(Some("Allow"), Some("Allow")),
            MicPermission::Allowed
        );
        assert_eq!(
            interpret_consent(Some("Deny"), Some("Allow")),
            MicPermission::DeniedSystemWide
        );
        assert_eq!(
            interpret_consent(Some("Allow"), Some("Deny")),
            MicPermission::DeniedForDesktopApps
        );
        assert!(MicPermission::DeniedForDesktopApps.is_denied());
        assert!(!MicPermission::Unknown.is_denied());
    }

    #[test]
    fn reading_this_machine_does_not_fail() {
        // Whatever the machine's setting is, reading it must not panic.
        let state = microphone_permission();
        eprintln!("microphone permission on this machine: {state:?}");
    }
}
