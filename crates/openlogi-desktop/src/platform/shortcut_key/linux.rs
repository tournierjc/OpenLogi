//! Passive `evdev` readers that record numpad key-downs for shortcut capture.

use std::sync::OnceLock;
use std::thread;

use evdev::{Device, EventSummary, KeyCode};
use tracing::warn;

use super::state::{PhysicalKey, record};

static STARTED: OnceLock<()> = OnceLock::new();

pub(super) fn start() {
    STARTED.get_or_init(|| {
        let devices = keyboard_devices();
        if devices.is_empty() {
            warn!("shortcut key probe found no readable keyboard devices");
            return;
        }
        for device in devices {
            thread::Builder::new()
                .name("openlogi-shortcut-key".into())
                .spawn(move || keyboard_thread(device))
                .map_err(|error| {
                    warn!(%error, "could not spawn shortcut key probe thread");
                })
                .ok();
        }
    });
}

fn keyboard_devices() -> Vec<Device> {
    evdev::enumerate()
        .filter_map(|(_path, device)| {
            let keys = device.supported_keys()?;
            if keys.contains(KeyCode::KEY_A) || keys.contains(KeyCode::KEY_KP0) {
                Some(device)
            } else {
                None
            }
        })
        .collect()
}

fn keyboard_thread(mut device: Device) {
    let path = device.name().unwrap_or("keyboard").to_owned();
    loop {
        match device.fetch_events() {
            Ok(events) => {
                for event in events {
                    if let EventSummary::Key(_event, key, value) = event.destructure()
                        && value == 1
                        && let Some(digit) = keypad_digit(key)
                    {
                        record(PhysicalKey::KeypadDigit(digit));
                    }
                }
            }
            Err(error) => {
                warn!(device = %path, %error, "shortcut key probe stopped reading");
                break;
            }
        }
    }
}

const fn keypad_digit(key: KeyCode) -> Option<u8> {
    match key {
        KeyCode::KEY_KP0 => Some(0),
        KeyCode::KEY_KP1 => Some(1),
        KeyCode::KEY_KP2 => Some(2),
        KeyCode::KEY_KP3 => Some(3),
        KeyCode::KEY_KP4 => Some(4),
        KeyCode::KEY_KP5 => Some(5),
        KeyCode::KEY_KP6 => Some(6),
        KeyCode::KEY_KP7 => Some(7),
        KeyCode::KEY_KP8 => Some(8),
        KeyCode::KEY_KP9 => Some(9),
        _ => None,
    }
}
