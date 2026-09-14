/**
 * Zentra Recording Console Controller
 * Coordinates recording lifecycle, countdown trigger, widget visibility, and session callbacks.
 */

import { invoke, listen, emit, isTauri } from './ipc.js';
import { state } from './state.js';
import { eventBus, Events } from './events.js';
import { initRecordingOptions, setOptionsDisabled, isAutoCountdownEnabled } from './recording/options.js';
import { initTelemetry, startTelemetry, stopTelemetry } from './recording/telemetry.js';
import { initMonitorsController } from './recording/monitors.js';

let btnRecord = null;
let recordBtnLabel = null;
let recordStateBadge = null;
let recordStateText = null;
let recordingCard = null;
let onSessionsUpdateCallback = null;

export function setSessionsUpdateCallback(fn) {
  onSessionsUpdateCallback = fn;
}

export function resetToIdleState() {
  state.isRecording = false;
  state.activeSessionId = null;
  stopTelemetry();

  if (btnRecord) {
    btnRecord.classList.remove('btn-recording');
    btnRecord.disabled = false;
  }
  if (recordBtnLabel) recordBtnLabel.textContent = 'Launch Recording Widget';
  if (recordStateBadge) recordStateBadge.classList.remove('recording');
  if (recordStateText) recordStateText.textContent = 'Ready to record';

  if (recordingCard) recordingCard.classList.remove('recording', 'is-recording');

  setOptionsDisabled(false);
  eventBus.emit(Events.RECORDING_STATE_CHANGED, { isRecording: false });
}

export async function toggleRecording() {
  if (!btnRecord) return;
  btnRecord.disabled = true;

  try {
    if (!state.isRecording) {
      // Open the floating widget in idle mode and hide main window
      await invoke('show_widget_window');
      await invoke('hide_main_window');

      // If auto-start countdown is enabled and timer > 0, begin countdown immediately!
      if (isAutoCountdownEnabled() && state.selectedCountdownTimer > 0) {
        if (isTauri) {
          setTimeout(() => {
            emit('zentra://start-countdown', {});
          }, 250);
        }
      }
    } else {
      // Stop Recording
      stopTelemetry();
      if (recordStateText) recordStateText.textContent = 'Finalizing segment & video...';

      await invoke('stop_recording');
      resetToIdleState();
      eventBus.emit(Events.SESSION_UPDATED);
      if (onSessionsUpdateCallback) {
        await onSessionsUpdateCallback();
      }
    }
  } catch (err) {
    console.error('Recording action error:', err);
    if (state.isRecording) {
      resetToIdleState();
    }
  } finally {
    if (btnRecord) btnRecord.disabled = false;
  }
}

export function initRecordingController() {
  btnRecord = document.getElementById('btn-record');
  recordBtnLabel = document.getElementById('record-btn-label');
  recordStateBadge = document.getElementById('record-state-badge');
  recordStateText = document.getElementById('record-state-text');
  recordingCard = document.querySelector('.recording-card');

  const timerDisplay = document.getElementById('timer-display');
  const statClicks = document.getElementById('stat-clicks');
  const statSegments = document.getElementById('stat-segments');

  // Initialize modular telemetry & options sub-controllers
  initTelemetry(timerDisplay, statClicks, statSegments);
  initRecordingOptions();
  initMonitorsController();

  if (btnRecord) {
    btnRecord.addEventListener('click', toggleRecording);
  }

  // Global F9 keyboard shortcut listener to stop recording from any window
  listen('zentra://request-stop-recording', async () => {
    if (state.isRecording) {
      await toggleRecording();
    }
  });

  // Listener for when recording starts (e.g. from floating widget Record button)
  listen('zentra://recording-started', () => {
    if (!state.isRecording) {
      state.isRecording = true;
      if (btnRecord) btnRecord.classList.add('btn-recording');
      if (recordBtnLabel) recordBtnLabel.textContent = 'Stop Recording';
      if (recordStateBadge) recordStateBadge.classList.add('recording');
      if (recordStateText) recordStateText.textContent = 'Recording Screen (via Widget)...';

      if (recordingCard) recordingCard.classList.add('recording', 'is-recording');

      setOptionsDisabled(true);
      startTelemetry();
      eventBus.emit(Events.RECORDING_STATE_CHANGED, { isRecording: true });
    }
  });

  // Listener for when recording stops (e.g. from floating widget Stop button)
  listen('zentra://recording-stopped', async () => {
    if (state.isRecording) {
      resetToIdleState();
      eventBus.emit(Events.SESSION_UPDATED);
      if (onSessionsUpdateCallback) {
        await onSessionsUpdateCallback();
      }
    }
  });
}