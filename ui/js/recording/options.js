/**
 * Zentra Recording Options & Settings Controller
 * Manages audio toggles, live magnifier, countdown delay chips, and localStorage persistence.
 */

import { emit, listen } from '../ipc.js';
import { state } from '../state.js';
import { setMonitorsDisabled } from './monitors.js';

let chkLiveMagnifier = null;
let chkSystemAudio = null;
let chkMicAudio = null;
let timerChips = [];
let chkAutoCountdown = null;

function updateTimerChipsUI(activeSecs) {
  timerChips.forEach((chip) => {
    const chipSecs = parseInt(chip.getAttribute('data-timer'), 10);
    if (chipSecs === activeSecs) {
      chip.classList.add('active');
    } else {
      chip.classList.remove('active');
    }
  });
}

export function initRecordingOptions() {
  chkLiveMagnifier = document.getElementById('chk-live-magnifier');
  chkSystemAudio = document.getElementById('chk-system-audio');
  chkMicAudio = document.getElementById('chk-mic-audio');
  timerChips = document.querySelectorAll('.btn-timer-chip');
  chkAutoCountdown = document.getElementById('chk-auto-countdown');

  // Countdown timer delay persistence & chip selection
  const savedTimer = localStorage.getItem('zentra_countdown_timer');
  if (savedTimer !== null) {
    const parsed = parseInt(savedTimer, 10);
    if ([0, 3, 5, 10].includes(parsed)) {
      state.selectedCountdownTimer = parsed;
    }
  }
  updateTimerChipsUI(state.selectedCountdownTimer);

  timerChips.forEach((chip) => {
    chip.addEventListener('click', () => {
      if (state.isRecording) return;
      const secs = parseInt(chip.getAttribute('data-timer'), 10);
      state.selectedCountdownTimer = secs;
      localStorage.setItem('zentra_countdown_timer', secs.toString());
      updateTimerChipsUI(secs);
      emit('zentra://timer-changed', { seconds: secs });
    });
  });

  // Auto-countdown persistence
  if (chkAutoCountdown) {
    const savedAuto = localStorage.getItem('zentra_auto_countdown');
    if (savedAuto !== null) {
      chkAutoCountdown.checked = savedAuto === 'true';
    }
    chkAutoCountdown.addEventListener('change', () => {
      localStorage.setItem('zentra_auto_countdown', chkAutoCountdown.checked);
    });
  }

  // Cross-window storage synchronization
  window.addEventListener('storage', (e) => {
    if (e.key === 'zentra_countdown_timer') {
      const val = parseInt(localStorage.getItem('zentra_countdown_timer'), 10);
      if ([0, 3, 5, 10].includes(val)) {
        state.selectedCountdownTimer = val;
        updateTimerChipsUI(val);
      }
    }
  });

  listen('zentra://timer-changed', (ev) => {
    if (ev && ev.payload && typeof ev.payload.seconds === 'number') {
      state.selectedCountdownTimer = ev.payload.seconds;
      updateTimerChipsUI(ev.payload.seconds);
    }
  });

  // Live Screen Magnifier settings
  if (chkLiveMagnifier) {
    const saved = localStorage.getItem('zentra_live_magnifier');
    if (saved !== null) {
      chkLiveMagnifier.checked = saved === 'true';
    }
    chkLiveMagnifier.addEventListener('change', () => {
      localStorage.setItem('zentra_live_magnifier', chkLiveMagnifier.checked);
    });
  }

  // System Audio Loopback settings
  if (chkSystemAudio) {
    const savedAudio = localStorage.getItem('zentra_record_system_audio');
    if (savedAudio !== null) {
      chkSystemAudio.checked = savedAudio === 'true';
    } else {
      chkSystemAudio.checked = true;
    }
    chkSystemAudio.addEventListener('change', () => {
      localStorage.setItem('zentra_record_system_audio', chkSystemAudio.checked);
    });
  }

  // Microphone Audio settings
  if (chkMicAudio) {
    const savedMic = localStorage.getItem('zentra_record_mic_audio');
    if (savedMic !== null) {
      chkMicAudio.checked = savedMic === 'true';
    } else {
      chkMicAudio.checked = true;
    }
    chkMicAudio.addEventListener('change', () => {
      localStorage.setItem('zentra_record_mic_audio', chkMicAudio.checked);
    });
  }
}

export function setOptionsDisabled(disabled) {
  if (chkLiveMagnifier) chkLiveMagnifier.disabled = disabled;
  if (chkSystemAudio) chkSystemAudio.disabled = disabled;
  if (chkMicAudio) chkMicAudio.disabled = disabled;
  timerChips.forEach((chip) => {
    chip.disabled = disabled;
  });
  if (chkAutoCountdown) chkAutoCountdown.disabled = disabled;
  setMonitorsDisabled(disabled);
}

export function isAutoCountdownEnabled() {
  return chkAutoCountdown ? chkAutoCountdown.checked : false;
}