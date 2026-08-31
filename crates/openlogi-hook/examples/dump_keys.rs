//! Dump every EV_KEY from a device for a fixed window.
use std::io::Write;
use std::time::{Duration, Instant};
use evdev::{Device, EventType};

fn main() {
    let path = std::env::args().nth(1).expect("path");
    let secs: u64 = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(20);
    let mut d = Device::open(&path).expect("open");
    println!("listening on {} ({:?}) for {secs}s", path, d.name());
    let _ = std::io::stdout().flush();
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end {
        match d.fetch_events() {
            Ok(iter) => {
                for ev in iter {
                    if ev.event_type() == EventType::KEY {
                        println!(
                            "KEY code={} ({:#x}) value={}",
                            ev.code(),
                            ev.code(),
                            ev.value()
                        );
                        let _ = std::io::stdout().flush();
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => eprintln!("fetch: {e}"),
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    println!("done");
}
