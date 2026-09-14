/**
 * Zentra Auto-Updater Service Component
 * Encapsulates update lifecycle, IPC checks, download streaming, and session safety guards.
 */

import { check } from '@tauri-apps/plugin-updater';
import { state } from '../state.js';

let activeUpdate = null;
let isChecking = false;
let isDownloading = false;
let updateReady = false;

export function getActiveUpdate() {
  return activeUpdate;
}

export function getIsChecking() {
  return isChecking;
}

export function getIsDownloading() {
  return isDownloading;
}

export function getIsUpdateReady() {
  return updateReady;
}

export async function performUpdateCheck() {
  if (isChecking || isDownloading) {
    return { skipped: true };
  }

  isChecking = true;
  try {
    const update = await check();
    if (update && update.available) {
      activeUpdate = update;
      return { success: true, hasUpdate: true, update };
    }
    activeUpdate = null;
    return { success: true, hasUpdate: false, update: null };
  } catch (err) {
    console.error('[Updater Service] Check failed:', err);
    return { success: false, error: err };
  } finally {
    isChecking = false;
  }
}

export async function performDownloadAndInstall({ onProgress, onStarted, onFinished }) {
  if (!activeUpdate || isDownloading) {
    return { started: false };
  }

  // Safety guard: prevent application restart during active screen recording
  if (state.isRecording) {
    throw new Error('A screen recording is currently active. Please stop recording before updating.');
  }

  isDownloading = true;
  let totalBytes = 0;
  let downloadedBytes = 0;

  try {
    await activeUpdate.downloadAndInstall((event) => {
      if (!event) return;
      switch (event.event) {
        case 'Started':
          totalBytes = event.data?.contentLength || 0;
          if (onStarted) onStarted(totalBytes);
          break;
        case 'Progress': {
          downloadedBytes += event.data?.chunkLength || 0;
          const percent = totalBytes > 0 ? Math.min(100, Math.round((downloadedBytes / totalBytes) * 100)) : 0;
          if (onProgress) onProgress({ percent, downloadedBytes, totalBytes });
          break;
        }
        case 'Finished':
          if (onFinished) onFinished();
          break;
        default:
          break;
      }
    });

    updateReady = true;
    return { success: true };
  } catch (err) {
    console.error('[Updater Service] Download/Install failed:', err);
    isDownloading = false;
    throw err;
  }
}