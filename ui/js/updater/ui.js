/**
 * Zentra Auto-Updater UI View Component
 * Manages DOM elements, badge states, update alert card, and download progress bar.
 */

let btnCheckUpdates = null;
let updateStatusText = null;
let updateAlertCard = null;
let updateVersionText = null;
let updateNotesText = null;
let btnUpdateNow = null;
let btnDismissUpdate = null;
let updateProgressContainer = null;
let updateProgressBar = null;
let updateProgressText = null;

let resetTimeoutId = null;

export function formatBytes(bytes) {
  if (!bytes || bytes <= 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

export function initUpdaterUI({ onCheckClick, onUpdateClick, onDismissClick }) {
  btnCheckUpdates = document.getElementById('btn-check-updates');
  updateStatusText = document.getElementById('update-status-text');
  updateAlertCard = document.getElementById('update-alert-card');
  updateVersionText = document.getElementById('update-version-text');
  updateNotesText = document.getElementById('update-notes-text');
  btnUpdateNow = document.getElementById('btn-update-now');
  btnDismissUpdate = document.getElementById('btn-dismiss-update');
  updateProgressContainer = document.getElementById('update-progress-container');
  updateProgressBar = document.getElementById('update-progress-bar');
  updateProgressText = document.getElementById('update-progress-text');

  if (btnCheckUpdates && onCheckClick) {
    btnCheckUpdates.addEventListener('click', onCheckClick);
  }

  if (btnUpdateNow && onUpdateClick) {
    btnUpdateNow.addEventListener('click', onUpdateClick);
  }

  if (btnDismissUpdate && onDismissClick) {
    btnDismissUpdate.addEventListener('click', onDismissClick);
  }
}

export function setCheckingState(isChecking) {
  if (!btnCheckUpdates) return;
  btnCheckUpdates.disabled = isChecking;
  if (isChecking) {
    btnCheckUpdates.classList.add('checking');
  } else {
    btnCheckUpdates.classList.remove('checking');
  }
}

export function setButtonStatusText(text, autoResetDelay = null) {
  if (resetTimeoutId) {
    clearTimeout(resetTimeoutId);
    resetTimeoutId = null;
  }

  if (updateStatusText) {
    updateStatusText.textContent = text;
  }

  if (autoResetDelay && autoResetDelay > 0) {
    resetTimeoutId = setTimeout(() => {
      if (updateStatusText) {
        updateStatusText.textContent = 'Check for Updates';
      }
      resetTimeoutId = null;
    }, autoResetDelay);
  }
}

export function setHasUpdateBadge(hasUpdate, versionText = '') {
  if (btnCheckUpdates) {
    if (hasUpdate) {
      btnCheckUpdates.classList.add('has-update');
    } else {
      btnCheckUpdates.classList.remove('has-update');
    }
  }

  if (hasUpdate && versionText) {
    setButtonStatusText(`Update ${versionText}`);
  }
}

export function showUpdateBanner(update) {
  if (!updateAlertCard || !update) return;

  if (updateVersionText) {
    updateVersionText.textContent = `v${update.version}`;
  }
  if (updateNotesText) {
    updateNotesText.textContent = update.body || 'A new version of Zentra is ready to install.';
  }

  updateAlertCard.hidden = false;
  updateAlertCard.classList.add('visible');
}

export function hideUpdateBanner() {
  if (!updateAlertCard) return;
  updateAlertCard.classList.remove('visible');
  setTimeout(() => {
    updateAlertCard.hidden = true;
  }, 250);
}

export function setDownloadingState(isDownloading, label = 'Downloading...') {
  if (btnUpdateNow) {
    btnUpdateNow.disabled = isDownloading;
    btnUpdateNow.textContent = label;
  }
  if (updateProgressContainer) {
    updateProgressContainer.hidden = !isDownloading;
  }
}

export function updateDownloadProgress(percent, downloadedBytes, totalBytes) {
  if (updateProgressBar) {
    updateProgressBar.style.width = `${percent}%`;
  }
  if (updateProgressText) {
    if (totalBytes > 0) {
      updateProgressText.textContent = `${formatBytes(downloadedBytes)} / ${formatBytes(totalBytes)} (${percent}%)`;
    } else {
      updateProgressText.textContent = `Downloaded ${formatBytes(downloadedBytes)}`;
    }
  }
}

export function setUpdateReady() {
  if (updateProgressBar) {
    updateProgressBar.style.width = '100%';
  }
  if (updateProgressText) {
    updateProgressText.textContent = 'Download complete. Finalizing update...';
  }
  if (btnUpdateNow) {
    btnUpdateNow.disabled = false;
    btnUpdateNow.textContent = 'Restarting...';
  }
}

export function setUpdateError(errorMessage) {
  setDownloadingState(false, 'Retry Update');
  if (updateProgressText) {
    updateProgressText.textContent = `Download error: ${errorMessage}`;
  }
}