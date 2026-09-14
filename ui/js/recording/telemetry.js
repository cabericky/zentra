/**
 * Zentra Recording Telemetry Controller
 * Periodically polls recording session metrics (elapsed timer, click count, segment count).
 */

import { invoke } from '../ipc.js';
import { formatSeconds } from '../formatters.js';
import { state } from '../state.js';

let timerDisplay = null;
let statClicks = null;
let statSegments = null;

export function initTelemetry(timerEl, clicksEl, segmentsEl) {
  timerDisplay = timerEl;
  statClicks = clicksEl;
  statSegments = segmentsEl;
}

export async function pollTelemetry() {
  if (!state.isRecording) return;
  try {
    const status = await invoke('get_recording_status');
    if (status && status.is_recording) {
      if (timerDisplay) timerDisplay.textContent = formatSeconds(status.elapsed_seconds);
      if (statClicks) statClicks.textContent = status.click_count.toLocaleString();
      if (statSegments) statSegments.textContent = status.segment_count.toLocaleString();
    }
  } catch (err) {
    console.error('Failed to poll recording status:', err);
  }
}

export function startTelemetry() {
  stopTelemetry();
  state.telemetryInterval = setInterval(pollTelemetry, 400);
}

export function stopTelemetry() {
  if (state.telemetryInterval) {
    clearInterval(state.telemetryInterval);
    state.telemetryInterval = null;
  }
}