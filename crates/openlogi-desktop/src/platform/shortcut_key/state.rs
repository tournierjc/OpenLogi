//! Shared timing state for correlating GPUI keystrokes with physical key events.

use std::borrow::Cow;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const MAX_AGE: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PhysicalKey {
    KeypadDigit(u8),
}

struct ProbeState {
    last: Option<(Instant, PhysicalKey)>,
}

static STATE: Mutex<ProbeState> = Mutex::new(ProbeState { last: None });

pub(super) fn record(physical: PhysicalKey) {
    if let Ok(mut state) = STATE.lock() {
        state.last = Some((Instant::now(), physical));
    }
}

/// GPUI Linux strips the `kp_` prefix from numpad digits, so `6` may mean
/// main-row `6` or `Kp6`. When a recent OS-level event says keypad, recover
/// the `kp` token [`KeyCombo::from_captured`] understands.
pub fn disambiguate(gpui_key: &str) -> Cow<'_, str> {
    let Ok(state) = STATE.lock() else {
        return Cow::Borrowed(gpui_key);
    };
    let Some((instant, physical)) = state.last else {
        return Cow::Borrowed(gpui_key);
    };
    if instant.elapsed() > MAX_AGE {
        return Cow::Borrowed(gpui_key);
    }
    match physical {
        PhysicalKey::KeypadDigit(digit) => {
            if gpui_key.len() == 1
                && gpui_key
                    .chars()
                    .next()
                    .is_some_and(|ch| ch.is_ascii_digit())
                && gpui_key.parse::<u8>() == Ok(digit)
            {
                return Cow::Owned(format!("kp{digit}"));
            }
        }
    }
    Cow::Borrowed(gpui_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_keypad_digit_disambiguates_matching_gpui_key() {
        record(PhysicalKey::KeypadDigit(6));
        assert_eq!(disambiguate("6").as_ref(), "kp6");
        assert_eq!(disambiguate("7").as_ref(), "7");
    }

    #[test]
    fn stale_keypad_events_do_not_disambiguate() {
        if let Ok(mut state) = STATE.lock() {
            state.last = Some((
                Instant::now()
                    .checked_sub(Duration::from_millis(250))
                    .unwrap_or_else(Instant::now),
                PhysicalKey::KeypadDigit(6),
            ));
        }
        assert_eq!(disambiguate("6").as_ref(), "6");
    }
}
