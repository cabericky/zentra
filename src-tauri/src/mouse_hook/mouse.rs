use crossbeam_channel::Sender;
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Performance::QueryPerformanceCounter;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, MSLLHOOKSTRUCT, WH_MOUSE_LL,
    WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_RBUTTONDOWN,
};
use super::types::RawClickEvent;

static mut HOOK_HANDLE: Option<HHOOK> = None;
static mut HOOK_SENDER: Option<Sender<RawClickEvent>> = None;

pub unsafe fn set_mouse_sender(sender: Option<Sender<RawClickEvent>>) {
    HOOK_SENDER = sender;
}

pub unsafe fn reset_mouse_state() {
    HOOK_SENDER = None;
}

pub unsafe fn install_mouse_hook() -> windows::core::Result<HHOOK> {
    let hook = SetWindowsHookExW(
        WH_MOUSE_LL,
        Some(low_level_mouse_proc),
        HINSTANCE(std::ptr::null_mut()),
        0,
    )?;
    HOOK_HANDLE = Some(hook);
    Ok(hook)
}

pub unsafe fn uninstall_mouse_hook() {
    let hook = HOOK_HANDLE;
    HOOK_HANDLE = None;
    if let Some(h) = hook {
        let _ = UnhookWindowsHookEx(h);
    }
}

pub unsafe extern "system" fn low_level_mouse_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let msg = w_param.0 as u32;
        let button = match msg {
            WM_LBUTTONDOWN => Some("left"),
            WM_RBUTTONDOWN => Some("right"),
            WM_MBUTTONDOWN => Some("middle"),
            _ => None,
        };

        if let Some(btn) = button {
            let mut qpc = 0i64;
            let _ = QueryPerformanceCounter(&mut qpc);

            let ms_hook = *(l_param.0 as *const MSLLHOOKSTRUCT);
            let raw_event = RawClickEvent {
                qpc,
                point: ms_hook.pt,
                button: btn,
            };

            if let Some(ref sender) = HOOK_SENDER {
                let _ = sender.send(raw_event);
            }
        }
    }

    CallNextHookEx(HOOK_HANDLE.unwrap_or_default(), n_code, w_param, l_param)
}