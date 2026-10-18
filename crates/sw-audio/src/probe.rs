//! Short diagnostics used by onboarding and settings: is the microphone
//! delivering audio, and can we capture what the computer plays?

use std::sync::{Arc, Mutex};
use std::time::Duration;

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};
use serde::Serialize;
use sw_core::AppError;
use ts_rs::TS;

use crate::devices::{DeviceDirection, audio_err, resolve};
use crate::dsp;
use crate::permission::microphone_permission;

/// Below this peak level a microphone is treated as delivering no signal.
const SILENT_PEAK_DBFS: f32 = -70.0;
/// Test tone used for the loopback self-test.
const TONE_HZ: f32 = 1000.0;
const TONE_AMPLITUDE: f32 = 0.2;
/// Minimum captured tone level for the loopback test to pass.
const TONE_DETECT_DBFS: f32 = -45.0;

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MicCheck {
    pub device_name: String,
    pub peak_dbfs: f32,
    pub rms_dbfs: f32,
    /// True when the device delivered audible signal during the check.
    pub heard_signal: bool,
    /// True when every captured sample was exactly zero, which usually means
    /// the OS is blocking access or the device is hardware-muted.
    pub all_zero: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LoopbackCheck {
    pub device_name: String,
    pub tone_detected: bool,
    pub tone_dbfs: f32,
    pub captured_ms: u32,
}

/// Everything a capture callback collected, already downmixed to mono.
#[derive(Debug, Default)]
struct Captured {
    mono: Vec<f32>,
}

fn build_capture<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sink: Arc<Mutex<Captured>>,
) -> Result<cpal::Stream, cpal::Error>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let channels = usize::from(config.channels);
    device.build_input_stream(
        *config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let floats: Vec<f32> = data.iter().map(|s| s.to_sample::<f32>()).collect();
            if let Ok(mut c) = sink.lock() {
                c.mono.extend(dsp::downmix(&floats, channels));
            }
        },
        |err| tracing::warn!(%err, "capture stream error"),
        None,
    )
}

fn open_capture(
    device: &cpal::Device,
    config: &cpal::SupportedStreamConfig,
    sink: Arc<Mutex<Captured>>,
) -> Result<cpal::Stream, AppError> {
    let stream_config = config.config();
    let result = match config.sample_format() {
        SampleFormat::F32 => build_capture::<f32>(device, &stream_config, sink),
        SampleFormat::I16 => build_capture::<i16>(device, &stream_config, sink),
        SampleFormat::I32 => build_capture::<i32>(device, &stream_config, sink),
        SampleFormat::U16 => build_capture::<u16>(device, &stream_config, sink),
        SampleFormat::U8 => build_capture::<u8>(device, &stream_config, sink),
        other => {
            return Err(AppError::Audio {
                detail: format!("unsupported sample format {other}"),
            });
        }
    };
    result.map_err(|err| match err.kind() {
        cpal::ErrorKind::PermissionDenied => AppError::MicrophonePermissionDenied,
        _ => audio_err("opening capture stream", err),
    })
}

fn take(sink: &Arc<Mutex<Captured>>) -> Vec<f32> {
    sink.lock()
        .map(|mut c| std::mem::take(&mut c.mono))
        .unwrap_or_default()
}

/// Records from the microphone for `duration` and reports levels.
pub fn check_microphone(device_id: Option<&str>, duration: Duration) -> Result<MicCheck, AppError> {
    if microphone_permission().is_denied() {
        return Err(AppError::MicrophonePermissionDenied);
    }
    let (device, device_name) = resolve(device_id, DeviceDirection::Input)?;
    let config = device
        .default_input_config()
        .map_err(|e| audio_err("reading microphone format", e))?;

    let sink = Arc::new(Mutex::new(Captured::default()));
    let stream = open_capture(&device, &config, Arc::clone(&sink))?;
    stream
        .play()
        .map_err(|e| audio_err("starting microphone", e))?;
    std::thread::sleep(duration);
    drop(stream);

    let mono = take(&sink);
    if mono.is_empty() {
        return Err(AppError::Audio {
            detail: format!("{device_name} delivered no audio frames"),
        });
    }
    let peak = dsp::peak(&mono);
    let peak_dbfs = dsp::to_dbfs(peak);
    Ok(MicCheck {
        device_name,
        peak_dbfs,
        rms_dbfs: dsp::to_dbfs(dsp::rms(&mono)),
        heard_signal: peak_dbfs > SILENT_PEAK_DBFS,
        all_zero: peak == 0.0,
    })
}

fn build_tone<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
) -> Result<cpal::Stream, cpal::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = usize::from(config.channels);
    #[allow(clippy::cast_precision_loss)]
    let step = 2.0 * std::f32::consts::PI * TONE_HZ / config.sample_rate as f32;
    let mut phase = 0.0_f32;
    device.build_output_stream(
        *config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let value = T::from_sample(TONE_AMPLITUDE * phase.sin());
                phase = (phase + step) % (2.0 * std::f32::consts::PI);
                for s in frame {
                    *s = value;
                }
            }
        },
        |err| tracing::warn!(%err, "tone stream error"),
        None,
    )
}

/// Plays a short 1 kHz tone on the output device while capturing that
/// device's loopback stream, and reports whether the tone was captured.
/// This proves the "them" side of a call can be recorded.
pub fn check_loopback(
    device_id: Option<&str>,
    duration: Duration,
) -> Result<LoopbackCheck, AppError> {
    let (device, device_name) = resolve(device_id, DeviceDirection::Output)?;
    let config = device
        .default_output_config()
        .map_err(|e| audio_err("reading output format", e))?;
    let sample_rate = config.sample_rate();

    let sink = Arc::new(Mutex::new(Captured::default()));
    let capture = open_capture(&device, &config, Arc::clone(&sink))?;
    let stream_config = config.config();
    let tone = match config.sample_format() {
        SampleFormat::F32 => build_tone::<f32>(&device, &stream_config),
        SampleFormat::I16 => build_tone::<i16>(&device, &stream_config),
        SampleFormat::I32 => build_tone::<i32>(&device, &stream_config),
        SampleFormat::U16 => build_tone::<u16>(&device, &stream_config),
        SampleFormat::U8 => build_tone::<u8>(&device, &stream_config),
        other => {
            return Err(AppError::Audio {
                detail: format!("unsupported sample format {other}"),
            });
        }
    }
    .map_err(|e| audio_err("opening test tone", e))?;

    capture
        .play()
        .map_err(|e| audio_err("starting system audio capture", e))?;
    tone.play().map_err(|e| audio_err("playing test tone", e))?;
    std::thread::sleep(duration);
    drop(tone);
    drop(capture);

    let mono = take(&sink);
    let tone_amp = dsp::goertzel_amplitude(&mono, sample_rate, TONE_HZ);
    let tone_dbfs = dsp::to_dbfs(tone_amp);
    let captured_ms =
        u32::try_from(mono.len() as u64 * 1000 / u64::from(sample_rate.max(1))).unwrap_or(u32::MAX);
    Ok(LoopbackCheck {
        device_name,
        tone_detected: tone_dbfs > TONE_DETECT_DBFS,
        tone_dbfs,
        captured_ms,
    })
}
