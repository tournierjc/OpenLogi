//! Low-level keyboard hook that records `VK_NUMPAD*` key-downs.
#![expect(
    unsafe_code,
    reason = "WH_KEYBOARD_LL is installed through the Win32 hook API"
)]

use std::sync::{OnceLock, mpsc};
use std::thread;

use tracing::warn;
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_NUMPAD0, VK_NUMPAD9,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, HC_ACTION, KBDLLHOOKSTRUCT,
    SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, WH_KEYBOARD_LL, WM_KEYDOWN,
    WM_SYSKEYDOWN,
};

use super::state::{PhysicalKey, record};

static STARTED: OnceLock<()> = OnceLock::new();

pub(super) fn start() {
    STARTED.get_or_init(|| {
        let (ready_tx, ready_rx) = mpsc::channel();
        thread::Builder::new()
            .name("openlogi-shortcut-key".into())
            .spawn(move || hook_thread(ready_tx))
            .map_err(|error| warn!(%error, "could not spawn shortcut key probe thread"))
            .ok();
        match ready_rx.recv() {
            Ok(Ok(())) => {}
            Ok(Err(message)) => warn!(%message, "shortcut key probe failed to install"),
            Err(error) => warn!(%error, "shortcut key probe thread exited before setup"),
        }
    });
}

fn hook_thread(ready_tx: mpsc::Sender<Result<(), String>>) {
    let hook = unsafe {
        SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_proc),
            std::ptr::null_mut(),
            0,
        )
    };
    if hook.is_null() {
        ready_tx
            .send(Err("SetWindowsHookExW(WH_KEYBOARD_LL) failed".into()))
            .ok();
        return;
    }
    ready_tx.send(Ok(())).ok();
    let _thread_id = unsafe { GetCurrentThreadId() };

    let mut msg = std::mem::MaybeUninit::uninit();
    while unsafe { GetMessageW(msg.as_mut_ptr(), std::ptr::null_mut(), 0, 0) } > 0 {
        let msg = unsafe { msg.assume_init() };
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    unsafe {
        UnhookWindowsHookEx(hook);
    }
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32
        && (wparam == WM_KEYDOWN as usize || wparam == WM_SYSKEYDOWN as usize)
    {
        let info = unsafe { &*(lparam as *const KBDLLHOOKSTRUCT) };
        if let Some(digit) = keypad_digit(VIRTUAL_KEY(info.vkCode as u16)) {
            record(PhysicalKey::KeypadDigit(digit));
        }
    }
    unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) }
}

const fn keypad_digit(vkey: VIRTUAL_KEY) -> Option<u8> {
    let code = vkey.0;
    if (VK_NUMPAD0.0..=VK_NUMPAD9.0).contains(&code) {
        Some((code - VK_NUMPAD0.0) as u8)
    } else {
        None
    }
}
