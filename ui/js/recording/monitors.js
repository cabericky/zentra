/**
 * Zentra Display & Monitor Selection Controller
 * Manages monitor enumeration, state synchronization, dropdown options,
 * and persistence for multi-monitor capture workflows.
 */

import { invoke } from '../ipc.js';
import { state } from '../state.js';
import { renderThumbnails, updateSelectedThumbnail, setThumbnailsDisabled } from './monitors/thumbnails.js';

let thumbnailsContainer = null;
let selectMonitor = null;
let btnRefresh = null;
let monitorCountBadge = null;
let statResolution = null;

export async function loadAvailableMonitors() {
  try {
    const monitors = await invoke('get_available_monitors');
    if (Array.isArray(monitors) && monitors.length > 0) {
      state.availableMonitors = monitors;
    } else {
      state.availableMonitors = [];
    }
    renderMonitorsUI();
  } catch (err) {
    console.error('Failed to enumerate connected monitors:', err);
  }
}

export function renderMonitorsUI() {
  const monitors = state.availableMonitors || [];

  // Update connected monitor count badge
  if (monitorCountBadge) {
    const count = monitors.length;
    monitorCountBadge.textContent = count === 1 ? '1 Display' : `${count} Displays`;
  }

  // Restore or default selected monitor
  const savedId = localStorage.getItem('zentra_selected_monitor');
  const exists = monitors.some((m) => m.id === savedId);
  if (savedId && exists) {
    state.selectedMonitorId = savedId;
  } else if (monitors.length > 0) {
    const primary = monitors.find((m) => m.is_primary) || monitors[0];
    state.selectedMonitorId = primary.id;
    localStorage.setItem('zentra_selected_monitor', primary.id);
  } else {
    state.selectedMonitorId = null;
  }

  // Populate Dropdown
  if (selectMonitor) {
    selectMonitor.innerHTML = '';
    if (monitors.length === 0) {
      const opt = document.createElement('option');
      opt.value = '';
      opt.textContent = 'Primary Display (Default)';
      selectMonitor.appendChild(opt);
    } else {
      monitors.forEach((m) => {
        const opt = document.createElement('option');
        opt.value = m.id;
        opt.textContent = m.name;
        if (m.id === state.selectedMonitorId) {
          opt.selected = true;
        }
        selectMonitor.appendChild(opt);
      });
    }
  }

  // Populate Visual Thumbnail Grid using modular renderer
  renderThumbnails(thumbnailsContainer, monitors, state.selectedMonitorId, (id) => {
    if (state.isRecording) return;
    selectDisplay(id);
  });

  updateResolutionTelemetry();
}

export function selectDisplay(monitorId) {
  state.selectedMonitorId = monitorId;
  localStorage.setItem('zentra_selected_monitor', monitorId);

  // Sync Dropdown
  if (selectMonitor) {
    selectMonitor.value = monitorId;
  }

  // Sync Visual Cards
  updateSelectedThumbnail(thumbnailsContainer, monitorId);

  updateResolutionTelemetry();
}

function updateResolutionTelemetry() {
  if (!statResolution) return;
  const monitors = state.availableMonitors || [];
  const selected = monitors.find((m) => m.id === state.selectedMonitorId);

  if (selected) {
    const shortLabel = selected.is_primary ? `Display (Primary)` : `Display (${selected.width}×${selected.height})`;
    statResolution.textContent = `${selected.width}×${selected.height} (${shortLabel}, 60 FPS)`;
  } else {
    statResolution.textContent = 'Primary Monitor (60 FPS)';
  }
}

export function getSelectedMonitorId() {
  return state.selectedMonitorId || null;
}

export function setMonitorsDisabled(disabled) {
  if (selectMonitor) selectMonitor.disabled = disabled;
  if (btnRefresh) btnRefresh.disabled = disabled;
  setThumbnailsDisabled(thumbnailsContainer, disabled);
}

export function initMonitorsController() {
  thumbnailsContainer = document.getElementById('monitor-thumbnails-grid');
  selectMonitor = document.getElementById('select-monitor');
  btnRefresh = document.getElementById('btn-refresh-monitors');
  monitorCountBadge = document.getElementById('monitor-count-badge');
  statResolution = document.getElementById('stat-resolution');

  if (selectMonitor) {
    selectMonitor.addEventListener('change', (e) => {
      if (state.isRecording) return;
      selectDisplay(e.target.value);
    });
  }

  if (btnRefresh) {
    btnRefresh.addEventListener('click', async () => {
      if (state.isRecording) return;
      btnRefresh.classList.add('rotating');
      await loadAvailableMonitors();
      setTimeout(() => {
        btnRefresh.classList.remove('rotating');
      }, 500);
    });
  }

  // Cross-window storage synchronization
  window.addEventListener('storage', (e) => {
    if (e.key === 'zentra_selected_monitor' && e.newValue) {
      if (e.newValue !== state.selectedMonitorId) {
        selectDisplay(e.newValue);
      }
    }
  });

  // Initial load
  loadAvailableMonitors();
}