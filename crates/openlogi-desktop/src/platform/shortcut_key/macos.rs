//! Local `NSEvent` monitor that records ANSI keypad key-downs.
#![expect(
    unsafe_code,
    reason = "NSEvent local monitors receive NonNull pointers from AppKit"
)]

use std::sync::OnceLock;

use objc2::MainThreadMarker;
use objc2_app_kit::{NSEvent, NSEventMask};
use tracing::warn;

use super::state::{PhysicalKey, record};

struct LocalKeyMonitor(
    #[expect(dead_code, reason = "held for process lifetime")]
    objc2::rc::Retained<objc2::runtime::AnyObject>,
);

static MONITOR: OnceLock<Option<LocalKeyMonitor>> = OnceLock::new();

pub(super) fn start() {
    MONITOR.get_or_init(|| {
        let _marker = MainThreadMarker::new()?;
        let handler: block2::RcBlock<dyn Fn(std::ptr::NonNull<NSEvent>) -> *mut NSEvent> =
            block2::RcBlock::new(|event| {
                let event_ref = unsafe { event.as_ref() };
                if let Some(digit) = keypad_digit(event_ref.keyCode()) {
                    record(PhysicalKey::KeypadDigit(digit));
                }
                event.as_ptr()
            });
        let monitor =
            NSEvent::addLocalMonitorForEventsMatchingMask_handler(NSEventMask::KeyDown, &handler)?;
        Some(LocalKeyMonitor(monitor))
    });
}

const fn keypad_digit(key_code: u16) -> Option<u8> {
    match key_code {
        0x52 => Some(0),
        0x53 => Some(1),
        0x54 => Some(2),
        0x55 => Some(3),
        0x56 => Some(4),
        0x57 => Some(5),
        0x58 => Some(6),
        0x59 => Some(7),
        0x5b => Some(8),
        0x5c => Some(9),
        _ => None,
    }
}
