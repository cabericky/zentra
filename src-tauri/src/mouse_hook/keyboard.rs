use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use crossbeam_channel::Sender;
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::Performance::QueryPerformanceCounter;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetCursorPos, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT,
    WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};
use super::types::RawClickEvent;

// Dynamic atomic key bindings (initialized with default VK codes)
static ZOOM_IN_VK: AtomicU32 = AtomicU32::new(0xC0);       // VK_OEM_3 (`~)
static ZOOM_OUT_VK: AtomicU32 = AtomicU32::new(0x1B);      // VK_ESCAPE
static STOP_RECORDING_VK: AtomicU32 = AtomicU32::new(0x78); // VK_F9

// Debounce state flags to prevent key repeat storms while held
static ZOOM_IN_DOWN: AtomicBool = AtomicBool::new(false);
static ZOOM_OUT_DOWN: AtomicBool = AtomicBool::new(false);
static STOP_RECORDING_DOWN: AtomicBool = AtomicBool::new(false);

static mut KBD_HOOK_HANDLE: Option<HHOOK> = None;
static mut KBD_SENDER: Option<Sender<RawClickEvent>> = None;

/// Updates the dynamic key bindings evaluated in the low-level keyboard hook callback.
pub fn update_hotkey_bindings(zoom_in_vk: u32, zoom_out_vk: u32, stop_recording_vk: u32) {
    ZOOM_IN_VK.store(zoom_in_vk, Ordering::SeqCst);
    ZOOM_OUT_VK.store(zoom_out_vk, Ordering::SeqCst);
    STOP_RECORDING_VK.store(stop_recording_vk, Ordering::SeqCst);
    ZOOM_IN_DOWN.store(false, Ordering::SeqCst);
    ZOOM_OUT_DOWN.store(false, Ordering::SeqCst);
    STOP_RECORDING_DOWN.store(false, Ordering::SeqCst);
}

pub unsafe fn set_keyboard_sender(sender: Option<Sender<RawClickEvent>>) {
    KBD_SENDER = sender;
}

pub unsafe fn reset_keyboard_state() {
    ZOOM_IN_DOWN.store(false, Ordering::SeqCst);
    ZOOM_OUT_DOWN.store(false, Ordering::SeqCst);
    STOP_RECORDING_DOWN.store(false, Ordering::SeqCst);
    KBD_SENDER = None;
}

pub unsafe fn install_keyboard_hook() -> windows::core::Result<HHOOK> {
    let hook = SetWindowsHookExW(
        WH_KEYBOARD_LL,
        Some(low_level_keyboard_proc),
        HINSTANCE(std::ptr::null_mut()),
        0,
    )?;
    KBD_HOOK_HANDLE = Some(hook);
    Ok(hook)
}

pub unsafe fn uninstall_keyboard_hook() {
    let hook = KBD_HOOK_HANDLE;
    KBD_HOOK_HANDLE = None;
    if let Some(h) = hook {
        let _ = UnhookWindowsHookEx(h);
    }
}

pub unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let msg = w_param.0 as u32;
        let kb = *(l_param.0 as *const KBDLLHOOKSTRUCT);

        let zoom_in_code = ZOOM_IN_VK.load(Ordering::Relaxed);
        let zoom_out_code = ZOOM_OUT_VK.load(Ordering::Relaxed);
        let stop_rec_code = STOP_RECORDING_VK.load(Ordering::Relaxed);

        if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
            if kb.vkCode == zoom_in_code {
                if !ZOOM_IN_DOWN.swap(true, Ordering::SeqCst) {
                    let mut qpc = 0i64;
                    let _ = QueryPerformanceCounter(&mut qpc);
                    let mut pt = POINT::default();
                    let _ = GetCursorPos(&mut pt);

                    let raw_event = RawClickEvent {
                        qpc,
                        point: pt,
                        button: "zoom_in",
                    };

                    if let Some(ref sender) = KBD_SENDER {
                        let _ = sender.send(raw_event);
                    }
                }
            } else if kb.vkCode == zoom_out_code {
                if !ZOOM_OUT_DOWN.swap(true, Ordering::SeqCst) {
                    let mut qpc = 0i64;
                    let _ = QueryPerformanceCounter(&mut qpc);
                    let mut pt = POINT::default();
                    let _ = GetCursorPos(&mut pt);

                    let raw_event = RawClickEvent {
                        qpc,
                        point: pt,
                        button: "zoom_out",
                    };

                    if let Some(ref sender) = KBD_SENDER {
                        let _ = sender.send(raw_event);
                    }
                }
            } else if kb.vkCode == stop_rec_code {
                if !STOP_RECORDING_DOWN.swap(true, Ordering::SeqCst) {
                    let mut qpc = 0i64;
                    let _ = QueryPerformanceCounter(&mut qpc);
                    let mut pt = POINT::default();
                    let _ = GetCursorPos(&mut pt);

                    let raw_event = RawClickEvent {
                        qpc,
                        point: pt,
                        button: "stop_recording",
                    };

                    if let Some(ref sender) = KBD_SENDER {
                        let _ = sender.send(raw_event);
                    }
                }
            }
        } else if msg == WM_KEYUP || msg == WM_SYSKEYUP {
            if kb.vkCode == zoom_in_code {
                ZOOM_IN_DOWN.store(false, Ordering::SeqCst);
            } else if kb.vkCode == zoom_out_code {
                ZOOM_OUT_DOWN.store(false, Ordering::SeqCst);
            } else if kb.vkCode == stop_rec_code {
                STOP_RECORDING_DOWN.store(false, Ordering::SeqCst);
            }
        }
    }

    CallNextHookEx(KBD_HOOK_HANDLE.unwrap_or_default(), n_code, w_param, l_param)
}