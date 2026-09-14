/**
 * Zentra Video Export Service
 * Manages export IPC invocation, progress event subscription, and file opening.
 */

import { invoke, listen } from '../ipc.js';
import { state } from '../state.js';

export async function openExportedFile(filePath) {
  if (!filePath) return;
  try {
    await invoke('open_file', { path: filePath });
  } catch (err) {
    console.error('Failed to reveal exported file:', err);
  }
}

export async function startExportSession(session, { onProgress }) {
  state.lastExportedPath = null;

  // Clean up previous event listener if any
  if (state.exportUnlisten) {
    state.exportUnlisten();
    state.exportUnlisten = null;
  }

  // Listen to real-time progress events
  state.exportUnlisten = await listen('zentra://export-progress', (event) => {
    const p = event.payload;
    if (p.session_id === session.id && onProgress) {
      const pct = Math.min(100, Math.max(0, Math.round(p.percent)));
      onProgress(pct, p.current_frame, p.total_frames, p.fps, p.status);
    }
  });

  const outputPath = await invoke('export_session', { sessionId: session.id });
  state.lastExportedPath = outputPath;
  return outputPath;
}