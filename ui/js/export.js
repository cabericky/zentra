/**
 * Zentra Smooth Zoom Export Controller
 * Coordinates video export lifecycle, UI telemetry view, and file actions.
 */

import { state } from './state.js';
import { eventBus, Events } from './events.js';
import {
  initExportUI,
  isCurrentlyExporting,
  setCurrentlyExporting,
  showExportCard,
  updateProgress,
  showCompleted,
  showFailed,
} from './export/ui.js';
import { startExportSession, openExportedFile } from './export/service.js';

let onSessionsUpdateCallback = null;

export function setExportSessionsCallback(fn) {
  onSessionsUpdateCallback = fn;
}

export async function triggerExport(session) {
  if (isCurrentlyExporting()) {
    alert('Another export is currently in progress. Please wait.');
    return;
  }

  showExportCard(session);

  try {
    const outputPath = await startExportSession(session, {
      onProgress: (pct, currentFrame, totalFrames, fps, status) => {
        updateProgress(pct, currentFrame, totalFrames, fps, status);
      },
    });

    showCompleted(outputPath);
    eventBus.emit(Events.EXPORT_COMPLETED, { session, outputPath });

    if (onSessionsUpdateCallback) {
      await onSessionsUpdateCallback();
    }
  } catch (err) {
    console.error('Export error:', err);
    showFailed(err?.message || err);
    alert('Export failed: ' + (err.message || err));
  } finally {
    setCurrentlyExporting(false);
  }
}

export function initExportController() {
  initExportUI({
    onOpenExportClick: async () => {
      if (state.lastExportedPath) {
        await openExportedFile(state.lastExportedPath);
      }
    },
  });
}