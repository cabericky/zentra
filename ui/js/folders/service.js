/**
 * Zentra Folder Management Backend Service
 * Encapsulates all IPC communications for designated folder operations.
 */

import { invoke } from '../ipc.js';

export async function fetchDesignatedFolderInfo() {
  return await invoke('get_designated_folder_info');
}

export async function fetchAvailableFolders() {
  return await invoke('list_available_folders');
}

export async function setDesignatedFolder(path) {
  return await invoke('set_designated_folder', { path });
}

export async function createDesignatedFolder(nameOrPath) {
  return await invoke('create_designated_folder', { nameOrPath });
}

export async function renameDesignatedFolder(oldPath, newName) {
  return await invoke('rename_designated_folder', { oldPath, newName });
}

export async function deleteDesignatedFolder(path) {
  return await invoke('delete_designated_folder', { path });
}