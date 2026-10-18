//! Command-line audio diagnostics, the same checks onboarding runs.
//!
//!     cargo run -p sw-audio --example audio_check
//!
//! Lists devices, records the default microphone for 2 s, and plays a short
//! test tone while capturing system audio.

use std::time::Duration;

fn main() {
    match sw_audio::list_devices() {
        Ok(devices) => {
            println!("Devices:");
            for d in devices {
                let mark = if d.is_default { "*" } else { " " };
                println!("  {mark} {:?} {}  [{}]", d.direction, d.name, d.id);
            }
        }
        Err(err) => println!("Listing devices failed: {err:?}"),
    }

    println!(
        "\nMicrophone permission: {:?}",
        sw_audio::microphone_permission()
    );
    println!("Recording 2 s from the default microphone, please speak...");
    match sw_audio::check_microphone(None, Duration::from_secs(2)) {
        Ok(check) => println!("Microphone: {check:#?}"),
        Err(err) => println!("Microphone check failed: {err:?}"),
    }

    println!("\nPlaying a 1 kHz test tone and capturing system audio...");
    match sw_audio::check_loopback(None, Duration::from_millis(1500)) {
        Ok(check) => println!("System audio: {check:#?}"),
        Err(err) => println!("System audio check failed: {err:?}"),
    }
}
