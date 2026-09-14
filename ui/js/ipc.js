/**
 * Zentra Tauri IPC Bridge
 * Wraps window.__TAURI__.core.invoke and window.__TAURI__.event.listen
 * with dev mock fallbacks for standalone browser testing.
 */

export const isTauri = typeof window !== 'undefined' && window.__TAURI__ !== undefined;

export const invoke = isTauri
  ? window.__TAURI__.core.invoke
  : async (cmd, args) => {
      console.warn('[Mock IPC] invoke:', cmd, args);
      if (cmd === 'list_sessions') return [];
      if (cmd === 'get_recording_status') {
        return { is_recording: false, elapsed_seconds: 0, click_count: 0, segment_count: 0 };
      }
      if (cmd === 'get_output_folder') return 'C:\\Users\\Videos\\Zentra';
      if (cmd === 'get_designated_folder_info') {
        return {
          current_path: 'C:\\Users\\Videos\\Zentra',
          current_name: 'Zentra',
          base_path: 'C:\\Users\\Videos\\Zentra',
          exists: true,
        };
      }
      if (cmd === 'list_available_folders') {
        return [
          { name: 'Zentra (Default)', path: 'C:\\Users\\Videos\\Zentra', is_active: true, exists: true, session_count: 0 }
        ];
      }
      return null;
    };

export const listen = isTauri
  ? window.__TAURI__.event.listen
  : async (eventName, cb) => {
      console.warn('[Mock IPC] listen:', eventName);
      return () => {};
    };

export const emit = isTauri && window.__TAURI__.event
  ? window.__TAURI__.event.emit
  : async (eventName, payload) => {
      console.warn('[Mock IPC] emit:', eventName, payload);
    };