//! Audio device discovery.

use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;
use sw_core::AppError;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DeviceDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AudioDevice {
    /// Stable identifier that can be stored in settings.
    pub id: String,
    pub name: String,
    pub direction: DeviceDirection,
    pub is_default: bool,
}

pub(crate) fn audio_err(context: &str, err: impl std::fmt::Display) -> AppError {
    AppError::Audio {
        detail: format!("{context}: {err}"),
    }
}

fn device_id(device: &cpal::Device) -> Option<String> {
    device.id().ok().map(|id| id.to_string())
}

fn device_name(device: &cpal::Device) -> String {
    device
        .description()
        .map(|d| d.name().to_owned())
        .unwrap_or_else(|_| "Unknown device".to_owned())
}

pub fn list_devices() -> Result<Vec<AudioDevice>, AppError> {
    let host = cpal::default_host();
    let default_in = host.default_input_device().as_ref().and_then(device_id);
    let default_out = host.default_output_device().as_ref().and_then(device_id);

    let mut out = Vec::new();
    for device in host
        .input_devices()
        .map_err(|e| audio_err("listing input devices", e))?
    {
        if let Some(id) = device_id(&device) {
            out.push(AudioDevice {
                is_default: default_in.as_deref() == Some(id.as_str()),
                name: device_name(&device),
                id,
                direction: DeviceDirection::Input,
            });
        }
    }
    for device in host
        .output_devices()
        .map_err(|e| audio_err("listing output devices", e))?
    {
        if let Some(id) = device_id(&device) {
            out.push(AudioDevice {
                is_default: default_out.as_deref() == Some(id.as_str()),
                name: device_name(&device),
                id,
                direction: DeviceDirection::Output,
            });
        }
    }
    Ok(out)
}

/// Resolves a stored device id, falling back to the system default when the
/// id is `None` or the device has disappeared (e.g. a headset was unplugged).
pub(crate) fn resolve(
    id: Option<&str>,
    direction: DeviceDirection,
) -> Result<(cpal::Device, String), AppError> {
    let host = cpal::default_host();
    if let Some(id) = id {
        match id.parse::<cpal::DeviceId>() {
            Ok(parsed) => {
                if let Some(device) = host.device_by_id(&parsed) {
                    let name = device_name(&device);
                    return Ok((device, name));
                }
                tracing::warn!(device = id, "stored audio device not found; using default");
            }
            Err(err) => tracing::warn!(device = id, %err, "unparseable device id; using default"),
        }
    }
    let device = match direction {
        DeviceDirection::Input => host.default_input_device(),
        DeviceDirection::Output => host.default_output_device(),
    }
    .ok_or(AppError::NoAudioDevice {
        kind: match direction {
            DeviceDirection::Input => "input",
            DeviceDirection::Output => "output",
        },
    })?;
    let name = device_name(&device);
    Ok((device, name))
}
