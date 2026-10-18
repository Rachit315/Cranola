//! Audio devices, capture and diagnostics. No Tauri dependency.

pub mod devices;
pub mod dsp;
pub mod permission;
pub mod probe;

pub use devices::{AudioDevice, DeviceDirection, list_devices};
pub use permission::{MicPermission, microphone_permission};
pub use probe::{LoopbackCheck, MicCheck, check_loopback, check_microphone};
