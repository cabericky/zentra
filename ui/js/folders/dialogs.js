/**
 * Zentra Folder Dialogs & Native Directory Explorer
 */

import { invoke } from '../ipc.js';

export async function browseForFolder() {
  try {
    return await invoke('browse_for_folder');
  } catch (err) {
    console.error('Failed to browse folder:', err);
    alert(`Browse error: ${err}`);
    return null;
  }
}

export async function openOutputFolder() {
  try {
    await invoke('open_output_folder');
  } catch (err) {
    console.error('Failed to open folder:', err);
  }
}