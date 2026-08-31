use std::io::Write;
use std::time::{Duration, Instant};
use evdev::{Device, EventType};

fn main() {
    let path = std::env::args().nth(1).expect("path");
    let mut d = Device::open(&path).expect("open");
    println!("listening on {} ({:?}) for 18s — press Profile Cycle now", path, d.name());
    let _ = std::io::stdout().flush();
    let end = Instant::now() + Duration::from_secs(18);
    while Instant::now() < end {
        match d.fetch_events() {
            Ok(iter) => {
                for ev in iter {
                    if ev.event_type() == EventType::KEY {
                        println!("KEY code={} ({:#x}) value={}", ev.code(), ev.code(), ev.value());
                        let _ = std::io::stdout().flush();
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => eprintln!("fetch: {e}"),
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    println!("done");
}
