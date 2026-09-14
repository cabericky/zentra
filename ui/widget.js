/**
 * Zentra Floating Overlay Widget Coordinator
 * Connects drag controller, countdown state machine, recording status polling, and IPC events.
 */

import { invoke, listen, emit, isTauri } from './js/ipc.js';
import { formatSeconds } from './js/formatters.js';
import { initWidgetDrag } from './js/widget/drag.js';
import { CountdownController } from './js/widget/countdown.js';

// DOM Elements
const timerEl = document.getElementById('widget-timer');
const dotEl = document.getElementById('widget-dot');
const btnAction = document.getElementById('btn-action');
const btnActionText = document.getElementById('btn-action-text');
const btnActionIcon = document.getElementById('btn-action-icon');
const btnReturn = document.getElementById('btn-return');
const btnTimer = document.getElementById('btn-timer');
const btnTimerText = document.getElementById('btn-timer-text');
const widgetHint = document.getElementById('widget-hint');
const pillEl = document.querySelector('.widget-pill');

// Widget State
let isRecording = false;
let pollInterval = null;
let stopKeyLabel = 'F9';

function applyHotkeys(hotkeys) {
  if (!hotkeys) return;
  if (hotkeys.stop_recording && hotkeys.stop_recording.label) {
    stopKeyLabel = hotkeys.stop_recording.label;
    if (widgetHint && (typeof countdown === 'undefined' || !countdown.isCountingDown)) {
      widgetHint.textContent = stopKeyLabel;
      widgetHint.title = `Stop shortcut (${stopKeyLabel})`;
    }
    if (btnAction && isRecording) {
      btnAction.title = `Stop Recording (${stopKeyLabel})`;
    }
  }
  if (hotkeys.zoom_out && typeof countdown !== 'undefined' && countdown) {
    countdown.setCancelKey(hotkeys.zoom_out.label, hotkeys.zoom_out.vk_code);
  }
}

// Initialize Dragging
initWidgetDrag(pillEl, invoke);

// State Transition Helpers
function setIdleState() {
  isRecording = false;
  if (dotEl) dotEl.classList.remove('recording', 'countdown');
  if (timerEl) {
    timerEl.classList.remove('countdown');
    timerEl.textContent = '00:00:00';
  }
  if (btnAction) {
    btnAction.className = 'btn-action btn-record';
    btnAction.title = 'Start Recording';
  }
  if (btnActionText) btnActionText.textContent = 'Record';
  if (btnActionIcon) btnActionIcon.innerHTML = '<use href="#icon-record-dot"></use>';
  if (btnReturn) btnReturn.classList.remove('hidden');
  if (btnTimer) btnTimer.classList.remove('hidden');
  if (widgetHint) {
    widgetHint.textContent = stopKeyLabel;
    widgetHint.title = `Stop shortcut (${stopKeyLabel})`;
  }
}

function setRecordingState(elapsedSec) {
  isRecording = true;
  if (dotEl) {
    dotEl.classList.remove('countdown');
    dotEl.classList.add('recording');
  }
  if (timerEl) {
    timerEl.classList.remove('countdown');
    timerEl.textContent = formatSeconds(elapsedSec);
  }
  if (btnAction) {
    btnAction.className = 'btn-action btn-stop';
    btnAction.title = `Stop Recording (${stopKeyLabel})`;
  }
  if (btnActionText) btnActionText.textContent = 'Stop';
  if (btnActionIcon) btnActionIcon.innerHTML = '<use href="#icon-stop-sq"></use>';
  if (btnReturn) btnReturn.classList.add('hidden');
  if (btnTimer) btnTimer.classList.add('hidden');
  if (widgetHint) {
    widgetHint.textContent = stopKeyLabel;
    widgetHint.title = `Stop shortcut (${stopKeyLabel})`;
  }
}

async function proceedToRecord() {
  try {
    const savedMag = localStorage.getItem('zentra_live_magnifier');
    const liveMagnifier = savedMag !== null ? savedMag === 'true' : true;
    const savedSysAudio = localStorage.getItem('zentra_record_system_audio');
    const recordSystemAudio = savedSysAudio !== null ? savedSysAudio === 'true' : true;
    const savedMicAudio = localStorage.getItem('zentra_record_mic_audio');
    const recordMic = savedMicAudio !== null ? savedMicAudio === 'true' : true;
    const savedMonitor = localStorage.getItem('zentra_selected_monitor');
    await invoke('start_recording', {
      debugMode: false,
      liveZoom: false,
      liveMagnifier,
      recordSystemAudio,
      recordMic,
      monitorId: savedMonitor || null,
    });
    setRecordingState(0);
  } catch (err) {
    console.error('Widget start_recording error:', err);
    setIdleState();
  }
}

// Initialize Countdown Controller
const countdown = new CountdownController({
  timerEl,
  dotEl,
  btnAction,
  btnActionText,
  btnActionIcon,
  btnReturn,
  btnTimer,
  btnTimerText,
  widgetHint,
  emit,
  onComplete: () => {
    proceedToRecord();
  },
  onStateChange: ({ isCountingDown }) => {
    if (!isCountingDown) {
      setIdleState();
    }
  },
});

// Telemetry & Status Polling
async function updateStatus() {
  try {
    const status = await invoke('get_recording_status');
    if (status && status.is_recording) {
      setRecordingState(status.elapsed_seconds);
    } else if (isRecording) {
      setIdleState();
    }
  } catch (e) {
    console.warn('Widget status poll warning:', e);
  }
}

function startPolling() {
  if (pollInterval) clearInterval(pollInterval);
  updateStatus();
  pollInterval = setInterval(updateStatus, 250);
}

startPolling();

// Toggle Action Button Handler
if (btnAction) {
  btnAction.addEventListener('click', async (e) => {
    e.stopPropagation();

    if (countdown.isCountingDown) {
      countdown.cancelCountdown();
      return;
    }

    btnAction.disabled = true;

    try {
      if (!isRecording) {
        countdown.startCountdown();
      } else {
        await invoke('stop_recording');
        setIdleState();
      }
    } catch (err) {
      console.error('Widget action error:', err);
    } finally {
      setTimeout(() => {
        btnAction.disabled = false;
      }, 300);
    }
  });
}

// Return to Main Window Button
if (btnReturn) {
  btnReturn.addEventListener('click', async (e) => {
    e.stopPropagation();
    if (countdown.isCountingDown) {
      countdown.cancelCountdown();
    }
    try {
      await invoke('hide_widget_window');
      await invoke('show_main_window');
    } catch (err) {
      console.error('Failed to return to main window:', err);
    }
  });
}

// Event Listeners from Tauri Backend
listen('zentra://recording-started', () => {
  setRecordingState(0);
  startPolling();
});

listen('zentra://recording-stopped', () => {
  setIdleState();
});

listen('zentra://timer-changed', (ev) => {
  if (ev && ev.payload && typeof ev.payload.seconds === 'number') {
    countdown.setTimerSeconds(ev.payload.seconds);
  }
});

listen('zentra://start-countdown', () => {
  if (!isRecording && !countdown.isCountingDown) {
    countdown.startCountdown();
  }
});

listen('zentra://hotkeys-updated', (ev) => {
  if (ev && ev.payload) {
    applyHotkeys(ev.payload);
  }
});

invoke('get_hotkeys').then((hotkeys) => {
  applyHotkeys(hotkeys);
}).catch(() => {});