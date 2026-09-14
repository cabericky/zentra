/**
 * Zentra Auto-Updater Feature Coordinator
 * Connects updater service, view component, and global event triggers.
 */

import {
  initUpdaterUI,
  setCheckingState,
  setButtonStatusText,
  setHasUpdateBadge,
  showUpdateBanner as renderUpdateBanner,
  hideUpdateBanner as closeUpdateBanner,
  setDownloadingState,
  updateDownloadProgress,
  setUpdateReady,
  setUpdateError,
} from './updater/ui.js';
import {
  performUpdateCheck,
  performDownloadAndInstall,
  getActiveUpdate,
  getIsUpdateReady,
} from './updater/service.js';

export function showUpdateBanner(update) {
  renderUpdateBanner(update);
}

export function hideUpdateBanner() {
  closeUpdateBanner();
}

export async function checkForUpdates({ silent = false } = {}) {
  setCheckingState(true);
  if (!silent) {
    setButtonStatusText('Checking...');
  }

  const result = await performUpdateCheck();

  setCheckingState(false);
  if (result.skipped) return;

  if (result.success && result.hasUpdate) {
    setHasUpdateBadge(true, `v${result.update.version}`);
    showUpdateBanner(result.update);
  } else if (result.success) {
    setHasUpdateBadge(false);
    if (!silent) {
      setButtonStatusText('Up to date', 3000);
    }
  } else {
    if (!silent) {
      setButtonStatusText('Update check error', 3500);
    }
  }
}

export async function downloadAndInstallUpdate() {
  const activeUpdate = getActiveUpdate();
  if (!activeUpdate) return;

  setDownloadingState(true, 'Downloading...');

  try {
    await performDownloadAndInstall({
      onStarted: () => {
        updateDownloadProgress(5, 0, 0);
      },
      onProgress: ({ percent, downloadedBytes, totalBytes }) => {
        updateDownloadProgress(percent, downloadedBytes, totalBytes);
      },
      onFinished: () => {
        setUpdateReady();
      },
    });
  } catch (err) {
    setUpdateError(err?.message || err);
  }
}

export function initUpdaterController() {
  initUpdaterUI({
    onCheckClick: () => {
      const activeUpdate = getActiveUpdate();
      if (activeUpdate && !getIsUpdateReady()) {
        showUpdateBanner(activeUpdate);
      } else {
        checkForUpdates({ silent: false });
      }
    },
    onUpdateClick: () => {
      downloadAndInstallUpdate();
    },
    onDismissClick: () => {
      hideUpdateBanner();
    },
  });

  // Seamless silent in-app background update check on app launch
  setTimeout(() => {
    checkForUpdates({ silent: true });
  }, 3000);
}