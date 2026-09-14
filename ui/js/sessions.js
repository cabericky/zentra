/**
 * Zentra Recorded Sessions History Controller
 * Coordinates session fetching, folder filtering, and view delegation.
 */

import { state } from './state.js';
import { eventBus, Events } from './events.js';
import { triggerExport } from './export.js';
import { fetchSessions } from './sessions/service.js';
import { renderSessionsList, renderFolderFilterDropdown } from './sessions/render.js';

let sessionsTotalCount = null;
let sessionsEmpty = null;
let sessionsList = null;
let filterSessionFolder = null;
let btnRefreshSessions = null;
let onAvailableFoldersUpdateCallback = null;

export function setSessionsFolderUpdateCallback(fn) {
  onAvailableFoldersUpdateCallback = fn;
}

export async function loadSessions() {
  const sessions = await fetchSessions();
  renderSessions(sessions);
  eventBus.emit(Events.SESSION_UPDATED, sessions);
  if (onAvailableFoldersUpdateCallback) {
    await onAvailableFoldersUpdateCallback();
  }
}

export function updateFolderFilter(availableFolders) {
  if (filterSessionFolder) {
    renderFolderFilterDropdown(filterSessionFolder, availableFolders, state.allSessions?.length || 0);
  }
}

export function renderSessions(sessions) {
  renderSessionsList(sessions, {
    sessionsTotalCount,
    sessionsEmpty,
    sessionsList,
    filterSessionFolder,
    onFolderFilterSelect: () => {
      renderSessions(state.allSessions);
    },
    onTriggerExport: (session) => {
      triggerExport(session);
    },
    onReloadSessions: async () => {
      await loadSessions();
    },
  });
}

export function initSessionsController() {
  sessionsTotalCount = document.getElementById('sessions-total-count');
  sessionsEmpty = document.getElementById('sessions-empty');
  sessionsList = document.getElementById('sessions-list');
  filterSessionFolder = document.getElementById('filter-session-folder');
  btnRefreshSessions = document.getElementById('btn-refresh-sessions');

  if (filterSessionFolder) {
    filterSessionFolder.addEventListener('change', (e) => {
      state.selectedFolderFilter = e.target.value;
      renderSessions(state.allSessions);
    });
  }

  if (btnRefreshSessions) {
    btnRefreshSessions.addEventListener('click', loadSessions);
  }

  // Reactive Domain Event Subscriptions
  eventBus.on(Events.AVAILABLE_FOLDERS_UPDATED, (availableFolders) => {
    updateFolderFilter(availableFolders);
  });

  eventBus.on(Events.FOLDER_CHANGED, async () => {
    await loadSessions();
  });

  eventBus.on(Events.EXPORT_COMPLETED, async () => {
    await loadSessions();
  });
}