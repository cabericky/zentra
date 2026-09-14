pub mod keyboard;
pub mod mouse;
pub mod types;
pub mod writer;

pub use types::ClickEvent;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use crossbeam_channel::{unbounded, Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PostThreadMessageW, MSG, WM_QUIT,
};

use crate::storage::Storage;
use types::RawClickEvent;

/// Controller for the active mouse and keyboard hook session.
pub struct MouseHookController {
    is_running: Arc<AtomicBool>,
    hook_thread: Option<JoinHandle<()>>,
    writer_thread: Option<JoinHandle<()>>,
    hook_thread_id: Arc<Mutex<u32>>,
    recent_clicks: Arc<RwLock<Vec<ClickEvent>>>,
}

impl MouseHookController {
    /// Starts low-level mouse and keyboard hooks on a background OS thread,
    /// and spawns a dedicated database writer thread.
    pub fn start(
        session_id: String,
        qpc_start: i64,
        qpc_frequency: i64,
        storage: Arc<Storage>,
        offset_x: i32,
        offset_y: i32,
        magnifier_callback: Option<Arc<dyn Fn(&str, i32, i32) + Send + Sync + 'static>>,
    ) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let (raw_sender, raw_receiver): (Sender<RawClickEvent>, Receiver<RawClickEvent>) = unbounded();
        let hook_thread_id = Arc::new(Mutex::new(0u32));
        let recent_clicks = Arc::new(RwLock::new(Vec::with_capacity(1000)));

        unsafe {
            mouse::set_mouse_sender(Some(raw_sender.clone()));
            keyboard::set_keyboard_sender(Some(raw_sender));
        }

        // Spawn Hook OS Thread with Win32 message loop
        let is_running_clone = Arc::clone(&is_running);
        let thread_id_clone = Arc::clone(&hook_thread_id);
        let hook_thread = thread::Builder::new()
            .name("zentra-mouse-hook".to_string())
            .spawn(move || {
                let tid = unsafe { windows::Win32::System::Threading::GetCurrentThreadId() };
                *thread_id_clone.lock() = tid;

                unsafe {
                    match mouse::install_mouse_hook() {
                        Ok(_) => log::info!("Low-level mouse hook installed on thread {}", tid),
                        Err(ref e) => log::error!("Failed to install WH_MOUSE_LL hook: {:?}", e),
                    }

                    match keyboard::install_keyboard_hook() {
                        Ok(_) => log::info!("Low-level keyboard hook installed on thread {}", tid),
                        Err(ref e) => log::error!("Failed to install WH_KEYBOARD_LL hook: {:?}", e),
                    }
                }

                let mut msg = MSG::default();
                // Standard Windows message pump required for low-level hooks
                while is_running_clone.load(Ordering::Relaxed) {
                    let ret = unsafe { GetMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0) };
                    if ret.0 <= 0 || msg.message == WM_QUIT {
                        break;
                    }
                    unsafe {
                        DispatchMessageW(&msg);
                    }
                }

                unsafe {
                    mouse::uninstall_mouse_hook();
                    keyboard::uninstall_keyboard_hook();
                }
                log::info!("Low-level mouse and keyboard hooks uninstalled.");
            })
            .expect("failed to spawn mouse/keyboard hook thread");

        // Spawn SQLite Writer Thread
        let writer_thread = writer::spawn_writer_thread(
            session_id,
            qpc_start,
            qpc_frequency,
            storage,
            offset_x,
            offset_y,
            raw_receiver,
            Arc::clone(&is_running),
            Arc::clone(&recent_clicks),
            magnifier_callback,
        );

        Self {
            is_running,
            hook_thread: Some(hook_thread),
            writer_thread: Some(writer_thread),
            hook_thread_id,
            recent_clicks,
        }
    }

    /// Retrieves a snapshot of recent clicks (useful for debug overlay and live zoom).
    pub fn get_recent_clicks(&self) -> Vec<ClickEvent> {
        self.recent_clicks.read().clone()
    }

    /// Stops the mouse/keyboard hooks and flushes remaining clicks.
    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);

        // Signal hook thread to quit message pump
        let tid = *self.hook_thread_id.lock();
        if tid != 0 {
            unsafe {
                let _ = PostThreadMessageW(tid, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        }

        unsafe {
            mouse::reset_mouse_state();
            keyboard::reset_keyboard_state();
        }

        if let Some(handle) = self.hook_thread.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.writer_thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for MouseHookController {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_mouse_hook_installation() {
        let clock = crate::clock::QpcClock::new();
        let storage = Arc::new(crate::storage::Storage::new(":memory:").unwrap());
        let mut controller = MouseHookController::start(
            "test_session".to_string(),
            clock.now(),
            clock.frequency,
            storage,
            0,
            0,
            None,
        );
        thread::sleep(Duration::from_millis(200));
        assert!(controller.is_running.load(Ordering::SeqCst));
        controller.stop();
    }
}