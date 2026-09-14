/**
 * Zentra Configurable Hotkeys Feature Coordinator
 * Orchestrates hotkey storage, dynamic evaluation, rebinding dialog, and main UI integration.
 */

import { invoke, listen } from './ipc.js';
import { state } from './state.js';
import { formatKeyLabel } from './hotkeys/formatter.js';
import { startRebind, cancelRebind, getActiveRebindAction } from './hotkeys/recorder.js';
import {
  initModalElements,
  getRebindControls,
  openModal,
  closeModal,
  isModalActive,
  updateModalBadges,
  validateDraftConflicts,
  showFeedback,
  setSaveDisabled,
} from './hotkeys/dialog.js';

export { formatKeyLabel };

// Main UI Elements
let displayZoomIn = null;
let displayZoomOut = null;
let displayStopRec = null;
let liveMagnifierDesc = null;
let btnOpenSettings = null;
let btnConfigureShortcuts = null;

// Controller State
let draftHotkeys = null;

/**
 * Updates shortcut badges and text descriptions on the main window.
 */
export function updateMainUI(hotkeys) {
  if (!hotkeys) return;

  if (displayZoomIn && hotkeys.zoom_in) {
    displayZoomIn.textContent = hotkeys.zoom_in.label.replace(' (Backtick)', '');
  }
  if (displayZoomOut && hotkeys.zoom_out) {
    displayZoomOut.textContent = hotkeys.zoom_out.label;
  }
  if (displayStopRec && hotkeys.stop_recording) {
    displayStopRec.textContent = hotkeys.stop_recording.label;
  }

  if (liveMagnifierDesc && hotkeys.zoom_in && hotkeys.zoom_out) {
    liveMagnifierDesc.textContent = `${hotkeys.zoom_in.label} = Zoom In & Follow Mouse • ${hotkeys.zoom_out.label} Key = Zoom Out to 1.0x.`;
  }
}

/**
 * Updates both main window UI and modal display elements.
 */
export function updateHotkeysUI(hotkeys) {
  if (!hotkeys) return;
  updateMainUI(hotkeys);
  updateModalBadges(hotkeys);
}

/**
 * Prepares draft configuration and opens the hotkey modal.
 */
export function openHotkeysModal() {
  draftHotkeys = state.hotkeys
    ? JSON.parse(JSON.stringify(state.hotkeys))
    : {
        zoom_in: { vk_code: 192, key: 'Backquote', label: '` (Backtick)' },
        zoom_out: { vk_code: 27, key: 'Escape', label: 'Esc' },
        stop_recording: { vk_code: 120, key: 'F9', label: 'F9' },
      };

  cancelRebind();
  openModal(draftHotkeys);
}

/**
 * Closes the hotkey modal and resets active recording state.
 */
export function closeHotkeysModal() {
  cancelRebind();
  closeModal();
}

/**
 * Loads current hotkeys configuration from backend SQLite settings.
 */
export async function loadHotkeys() {
  try {
    const hotkeys = await invoke('get_hotkeys');
    if (hotkeys) {
      state.hotkeys = hotkeys;
      updateHotkeysUI(hotkeys);
    }
    return hotkeys;
  } catch (err) {
    console.error('Failed to load hotkeys from storage:', err);
    return null;
  }
}

/**
 * Saves draft hotkeys to backend SQLite settings.
 */
export async function saveHotkeys() {
  if (!validateDraftConflicts(draftHotkeys)) return;

  try {
    setSaveDisabled(true);
    const saved = await invoke('save_hotkeys', { hotkeys: draftHotkeys });
    state.hotkeys = saved;
    updateHotkeysUI(saved);
    showFeedback('Hotkeys updated and applied successfully!', 'success');
    setTimeout(() => {
      closeHotkeysModal();
      setSaveDisabled(false);
    }, 600);
  } catch (err) {
    console.error('Failed to save hotkeys:', err);
    showFeedback(typeof err === 'string' ? err : 'Failed to save hotkeys', 'error');
    setSaveDisabled(false);
  }
}

/**
 * Resets hotkeys to factory defaults.
 */
export async function resetHotkeys() {
  try {
    const defaults = await invoke('reset_hotkeys');
    state.hotkeys = defaults;
    draftHotkeys = JSON.parse(JSON.stringify(defaults));
    updateHotkeysUI(defaults);
    validateDraftConflicts(draftHotkeys);
    showFeedback('Hotkeys restored to default settings.', 'success');
  } catch (err) {
    console.error('Failed to reset hotkeys:', err);
    showFeedback('Failed to reset hotkeys.', 'error');
  }
}

/**
 * Initializes hotkey controller event bindings and modal dialog.
 */
export function initHotkeysController() {
  btnOpenSettings = document.getElementById('btn-open-settings');
  btnConfigureShortcuts = document.getElementById('btn-configure-shortcuts');
  displayZoomIn = document.getElementById('shortcut-display-zoom-in');
  displayZoomOut = document.getElementById('shortcut-display-zoom-out');
  displayStopRec = document.getElementById('shortcut-display-stop-recording');
  liveMagnifierDesc = document.getElementById('live-magnifier-hotkey-desc');

  // Initialize dialog DOM bindings
  initModalElements({
    onSave: saveHotkeys,
    onReset: resetHotkeys,
    onClose: closeHotkeysModal,
  });

  const { rebindButtons, labelElements } = getRebindControls();

  ['zoom_in', 'zoom_out', 'stop_recording'].forEach((act) => {
    if (rebindButtons[act]) {
      rebindButtons[act].addEventListener('click', (e) => {
        e.stopPropagation();
        startRebind({
          action: act,
          rebindButtons,
          labelElements,
          onCapture: (capturedAction, binding) => {
            if (draftHotkeys) {
              draftHotkeys[capturedAction] = binding;
              updateModalBadges(draftHotkeys);
              validateDraftConflicts(draftHotkeys);
            }
          },
          onFeedback: (msg, type) => showFeedback(msg, type),
        });
      });
    }
  });

  // Modal open triggers
  if (btnOpenSettings) {
    btnOpenSettings.addEventListener('click', openHotkeysModal);
  }
  if (btnConfigureShortcuts) {
    btnConfigureShortcuts.addEventListener('click', openHotkeysModal);
  }

  // Global keydown modal close on Esc when not actively rebinding
  window.addEventListener('keydown', (e) => {
    if (isModalActive()) {
      if (e.key === 'Escape' && !getActiveRebindAction()) {
        closeHotkeysModal();
      }
    }
  });

  // Listen for hotkeys updated event across windows / tray
  listen('zentra://hotkeys-updated', (ev) => {
    if (ev && ev.payload) {
      state.hotkeys = ev.payload;
      updateHotkeysUI(ev.payload);
    }
  });
}