/**
 * Zentra Frontend Main Coordinator
 * Coordinates feature controllers, state sync, and system event listeners.
 */

import { invoke, listen } from './js/ipc.js';
import { state } from './js/state.js';
import { initIcons } from './js/icons.js';
import { initRecordingController } from './js/recording.js';
import { loadAvailableMonitors } from './js/recording/monitors.js';
import { initFolderController, loadDesignatedFolder, loadAvailableFolders } from './js/folders.js';
import { initSessionsController, loadSessions } from './js/sessions.js';
import { initExportController } from './js/export.js';
import { initHotkeysController, loadHotkeys, openHotkeysModal } from './js/hotkeys.js';
import { initUpdaterController } from './js/updater.js';

async function init() {
  // 0. Load Modular SVG Icon Sprites
  await initIcons();

  // 1. Initialize Modular Feature Controllers
  initRecordingController();
  initFolderController();
  initSessionsController();
  initExportController();
  initHotkeysController();
  initUpdaterController();

  // 2. Window & Shortcut Listeners
  window.addEventListener('focus', () => {
    if (!state.isRecording) {
      loadSessions();
      loadDesignatedFolder();
      loadAvailableMonitors();
      loadHotkeys();
    }
  });

  window.addEventListener('keydown', (e) => {
    const isZoomOutKey = e.key === 'Escape' || (state.hotkeys && e.keyCode === state.hotkeys.zoom_out?.vk_code);
    if (isZoomOutKey) {
      invoke('reset_screen_magnifier').catch(() => {});
    }
  });

  listen('zentra://focus-settings', () => {
    openHotkeysModal();
  });

  // 3. Initial Data Fetch
  await loadDesignatedFolder();
  await loadAvailableFolders();
  await loadSessions();
  await loadHotkeys();
}

// Start on DOM ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}