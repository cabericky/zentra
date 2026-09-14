/**
 * Zentra Central Application State
 * Single source of truth for runtime frontend state.
 */

export const state = {
  isRecording: false,
  activeSessionId: null,
  telemetryInterval: null,
  exportUnlisten: null,
  lastExportedPath: null,
  currentDesignatedFolder: null,
  availableFolders: [],
  allSessions: [],
  selectedFolderFilter: '__ALL__',
  selectedCountdownTimer: 3,
  availableMonitors: [],
  selectedMonitorId: null,
  hotkeys: null,
};