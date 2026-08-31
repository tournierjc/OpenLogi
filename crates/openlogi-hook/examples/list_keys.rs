//! List mouse button key codes exposed by a Logitech evdev node.
use evdev::{Device, KeyCode};

fn main() {
    let path = std::env::args().nth(1).expect("usage: list_keys /dev/input/eventN");
    let device = Device::open(path).expect("open device");
    println!("device: {}", device.name().unwrap_or("unnamed"));
    let Some(keys) = device.supported_keys() else {
        println!("no keys");
        return;
    };
    for key in keys.iter() {
        if key.code() >= KeyCode::BTN_LEFT.code() {
            println!("{key:?} ({:#06x})", key.code());
        }
    }
}
