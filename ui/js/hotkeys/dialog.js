/**
 * Zentra Hotkeys Modal Dialog & Feedback Controller
 * Manages modal visibility, form state, conflict detection, and feedback messaging.
 */

let modalBackdrop = null;
let btnCloseModal = null;
let btnCancelModal = null;
let btnSaveHotkeys = null;
let btnResetHotkeys = null;
let feedbackEl = null;

const rebindButtons = {};
const badgeElements = {};
const labelElements = {};
const itemCardElements = {};

/**
 * Initializes modal DOM elements and binds close triggers.
 */
export function initModalElements({ onSave, onReset, onClose }) {
  modalBackdrop = document.getElementById('hotkeys-modal-backdrop');
  btnCloseModal = document.getElementById('btn-close-hotkeys-modal');
  btnCancelModal = document.getElementById('btn-cancel-hotkeys');
  btnSaveHotkeys = document.getElementById('btn-save-hotkeys');
  btnResetHotkeys = document.getElementById('btn-reset-hotkeys');
  feedbackEl = document.getElementById('hotkeys-feedback');

  ['zoom_in', 'zoom_out', 'stop_recording'].forEach((act) => {
    rebindButtons[act] = document.getElementById(`btn-rebind-${act.replace('_', '-')}`);
    badgeElements[act] = document.getElementById(`hotkey-badge-${act.replace('_', '-')}`);
    labelElements[act] = document.getElementById(`hotkey-label-${act.replace('_', '-')}`);
    itemCardElements[act] = document.getElementById(`card-hotkey-${act.replace('_', '-')}`);
  });

  if (btnCloseModal) {
    btnCloseModal.addEventListener('click', () => onClose && onClose());
  }
  if (btnCancelModal) {
    btnCancelModal.addEventListener('click', () => onClose && onClose());
  }
  if (btnSaveHotkeys) {
    btnSaveHotkeys.addEventListener('click', () => onSave && onSave());
  }
  if (btnResetHotkeys) {
    btnResetHotkeys.addEventListener('click', () => onReset && onReset());
  }

  if (modalBackdrop) {
    modalBackdrop.addEventListener('click', (e) => {
      if (e.target === modalBackdrop && onClose) {
        onClose();
      }
    });
  }
}

/**
 * Returns references to modal rebinding buttons and labels.
 */
export function getRebindControls() {
  return {
    rebindButtons,
    labelElements,
    badgeElements,
    itemCardElements,
  };
}

/**
 * Opens the hotkeys modal backdrop.
 */
export function openModal(draftHotkeys) {
  if (!modalBackdrop) return;
  hideFeedback();
  updateModalBadges(draftHotkeys);
  validateDraftConflicts(draftHotkeys);

  modalBackdrop.classList.add('active');
  modalBackdrop.setAttribute('aria-hidden', 'false');
}

/**
 * Closes the hotkeys modal backdrop.
 */
export function closeModal() {
  if (!modalBackdrop) return;
  modalBackdrop.classList.remove('active');
  modalBackdrop.setAttribute('aria-hidden', 'true');
}

/**
 * Returns whether modal dialog is actively visible.
 */
export function isModalActive() {
  return modalBackdrop ? modalBackdrop.classList.contains('active') : false;
}

/**
 * Updates kbd badges and text labels inside modal cards.
 */
export function updateModalBadges(hotkeys) {
  if (!hotkeys) return;

  ['zoom_in', 'zoom_out', 'stop_recording'].forEach((act) => {
    const binding = hotkeys[act];
    if (binding) {
      if (badgeElements[act]) {
        badgeElements[act].textContent = binding.label.replace(' (Backtick)', '');
      }
      if (labelElements[act]) {
        labelElements[act].textContent = binding.label;
      }
    }
  });
}

/**
 * Checks draft bindings for duplicate keycodes and highlights conflict cards.
 */
export function validateDraftConflicts(draftHotkeys) {
  if (!draftHotkeys) return true;

  const actions = ['zoom_in', 'zoom_out', 'stop_recording'];
  actions.forEach((act) => {
    if (itemCardElements[act]) {
      itemCardElements[act].classList.remove('has-conflict');
    }
  });

  const conflicts = [];
  if (draftHotkeys.zoom_in.vk_code === draftHotkeys.zoom_out.vk_code) {
    conflicts.push('Zoom In and Zoom Out cannot use the same key.');
    if (itemCardElements['zoom_in']) itemCardElements['zoom_in'].classList.add('has-conflict');
    if (itemCardElements['zoom_out']) itemCardElements['zoom_out'].classList.add('has-conflict');
  }
  if (draftHotkeys.zoom_in.vk_code === draftHotkeys.stop_recording.vk_code) {
    conflicts.push('Zoom In and Stop Recording cannot use the same key.');
    if (itemCardElements['zoom_in']) itemCardElements['zoom_in'].classList.add('has-conflict');
    if (itemCardElements['stop_recording']) itemCardElements['stop_recording'].classList.add('has-conflict');
  }
  if (draftHotkeys.zoom_out.vk_code === draftHotkeys.stop_recording.vk_code) {
    conflicts.push('Zoom Out and Stop Recording cannot use the same key.');
    if (itemCardElements['zoom_out']) itemCardElements['zoom_out'].classList.add('has-conflict');
    if (itemCardElements['stop_recording']) itemCardElements['stop_recording'].classList.add('has-conflict');
  }

  if (conflicts.length > 0) {
    showFeedback(conflicts[0], 'error');
    setSaveDisabled(true);
    return false;
  }

  hideFeedback();
  setSaveDisabled(false);
  return true;
}

export function showFeedback(text, type = 'error') {
  if (!feedbackEl) return;
  feedbackEl.textContent = text;
  feedbackEl.className = `hotkeys-feedback-msg ${type}`;
  feedbackEl.style.display = 'block';
}

export function hideFeedback() {
  if (!feedbackEl) return;
  feedbackEl.textContent = '';
  feedbackEl.className = 'hotkeys-feedback-msg';
  feedbackEl.style.display = 'none';
}

export function setSaveDisabled(disabled) {
  if (btnSaveHotkeys) {
    btnSaveHotkeys.disabled = disabled;
  }
}