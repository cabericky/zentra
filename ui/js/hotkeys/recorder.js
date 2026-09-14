/**
 * Zentra Hotkey Keypress Recorder & Capture Engine
 * Handles interactive key capture, modifier prevention, and key recording state.
 */

import { formatKeyLabel } from './formatter.js';

let activeAction = null;
let currentButtons = {};
let currentLabels = {};
let onCaptureCallback = null;
let onFeedbackCallback = null;

function handleKeyDown(e) {
  if (!activeAction) return;

  e.preventDefault();
  e.stopPropagation();

  // Ignore bare modifier key presses
  if (['Shift', 'Control', 'Alt', 'Meta'].includes(e.key)) {
    if (onFeedbackCallback) {
      onFeedbackCallback('Please press a standard key (e.g. F-keys, letters, punctuation).', 'error');
    }
    return;
  }

  const action = activeAction;
  const vkCode = e.keyCode || e.which;
  const keyName = e.key;
  const code = e.code;
  const label = formatKeyLabel(vkCode, keyName, code);

  const binding = {
    vk_code: vkCode,
    key: code || keyName,
    label,
  };

  const cb = onCaptureCallback;
  cancelRebind();

  if (cb) {
    cb(action, binding);
  }
}

/**
 * Initiates key recording for a designated hotkey action.
 */
export function startRebind({
  action,
  rebindButtons,
  labelElements,
  onCapture,
  onFeedback,
}) {
  cancelRebind();

  activeAction = action;
  currentButtons = rebindButtons || {};
  currentLabels = labelElements || {};
  onCaptureCallback = onCapture;
  onFeedbackCallback = onFeedback;

  const btn = currentButtons[action];
  if (btn) {
    btn.classList.add('recording');
    if (currentLabels[action]) {
      currentLabels[action].textContent = 'Press any key...';
    }
  }

  window.addEventListener('keydown', handleKeyDown, true);
}

/**
 * Cancels active key recording and restores previous label.
 */
export function cancelRebind(fallbackBinding = null) {
  if (activeAction && currentButtons[activeAction]) {
    currentButtons[activeAction].classList.remove('recording');
    if (fallbackBinding && currentLabels[activeAction]) {
      currentLabels[activeAction].textContent = fallbackBinding.label;
    }
  }

  activeAction = null;
  onCaptureCallback = null;
  onFeedbackCallback = null;
  window.removeEventListener('keydown', handleKeyDown, true);
}

/**
 * Returns currently active recording action (or null).
 */
export function getActiveRebindAction() {
  return activeAction;
}