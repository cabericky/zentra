/**
 * Zentra Recorded Sessions Item Actions
 * Implements folder opening, video playback, file renaming, and deletion.
 */

import { invoke } from '../ipc.js';

export async function openSessionFolder(outputDir) {
  try {
    await invoke('open_folder', { path: outputDir });
  } catch (err) {
    console.error('Failed to open session folder:', err);
  }
}

export async function playZoomVideo(outputDir, zoomName) {
  try {
    const filePath = `${outputDir}\\${zoomName}`;
    await invoke('open_file', { path: filePath });
  } catch (err) {
    console.error('Failed to open zoom video file:', err);
  }
}

export async function renameSession(session, segName, onReload) {
  const curBaseName = segName.replace(/\.mp4$/i, '');
  const newName = prompt(`Enter new file name for session video:\n(Current: ${segName})`, curBaseName);
  if (newName === null) return;
  const trimmed = newName.trim();
  if (!trimmed || trimmed === curBaseName) return;

  try {
    await invoke('rename_session', { sessionId: session.id, newName: trimmed });
    if (onReload) await onReload();
  } catch (err) {
    console.error('Failed to rename session:', err);
    alert(`Rename failed: ${err}`);
  }
}

export async function deleteSession(session, segName, onReload) {
  if (confirm(`Are you sure you want to permanently delete session "${segName}" and its video files from disk?`)) {
    try {
      await invoke('delete_session', { sessionId: session.id });
      if (onReload) await onReload();
    } catch (err) {
      console.error('Failed to delete session:', err);
      alert(`Delete failed: ${err}`);
    }
  }
}