/**
 * Zentra Designated Save Folder Coordinator
 * Orchestrates folder state, service IPC, view updates, and panel dialogs.
 */

import { state } from './state.js';
import { eventBus, Events } from './events.js';
import { browseForFolder, openOutputFolder } from './folders/dialogs.js';
import {
  initPanels,
  showFolderFeedback,
  showRenameFolderFeedback,
  toggleCreatePanel,
  hideCreatePanel,
  toggleRenamePanel,
  hideRenamePanel,
  hideAllPanels,
} from './folders/panels.js';
import {
  fetchDesignatedFolderInfo,
  fetchAvailableFolders,
  setDesignatedFolder,
  createDesignatedFolder,
  renameDesignatedFolder,
  deleteDesignatedFolder,
} from './folders/service.js';
import {
  initFolderView,
  renderDesignatedFolderInfo,
  renderFolderDropdown,
} from './folders/view.js';

export { showFolderFeedback, showRenameFolderFeedback };

let selectDesignatedFolder = null;
let inputNewFolderName = null;
let btnSubmitCreateFolder = null;
let inputRenameFolderName = null;
let btnSubmitRenameFolder = null;
let btnDeleteFolder = null;

// Legacy callback support (delegated to eventBus)
let onSessionsUpdateCallback = null;
export function setFolderSessionsCallback(fn) {
  onSessionsUpdateCallback = fn;
}

function notifyFolderChanged() {
  eventBus.emit(Events.FOLDER_CHANGED, state.currentDesignatedFolder);
  if (onSessionsUpdateCallback) {
    onSessionsUpdateCallback();
  }
}

export async function loadDesignatedFolder() {
  try {
    const info = await fetchDesignatedFolderInfo();
    if (info) {
      state.currentDesignatedFolder = info;
      renderDesignatedFolderInfo(info);
    }
  } catch (err) {
    console.error('Failed to get designated folder info:', err);
  }
}

export async function loadAvailableFolders() {
  try {
    const folders = await fetchAvailableFolders();
    state.availableFolders = folders || [];

    renderFolderDropdown(state.availableFolders);
    eventBus.emit(Events.AVAILABLE_FOLDERS_UPDATED, state.availableFolders);
  } catch (err) {
    console.error('Failed to list available folders:', err);
  }
}

export async function handleFolderSelectChange(path) {
  if (!path) return;
  try {
    const info = await setDesignatedFolder(path);
    if (info) {
      state.currentDesignatedFolder = info;
      showFolderFeedback(`Selected folder: ${info.current_name}`, 'success');
      await loadDesignatedFolder();
      await loadAvailableFolders();
      notifyFolderChanged();
    }
  } catch (err) {
    console.error('Failed to set designated folder:', err);
    showFolderFeedback(`Error: ${err}`, 'error');
    alert(`Could not select folder: ${err}`);
    await loadAvailableFolders();
  }
}

export async function handleCreateFolder() {
  const rawName = inputNewFolderName ? inputNewFolderName.value.trim() : '';
  if (!rawName) {
    showFolderFeedback('Please enter a valid folder name.', 'error');
    if (inputNewFolderName) inputNewFolderName.focus();
    return;
  }

  try {
    if (btnSubmitCreateFolder) btnSubmitCreateFolder.disabled = true;
    const info = await createDesignatedFolder(rawName);
    if (info) {
      showFolderFeedback(`Folder "${info.current_name}" created & designated!`, 'success');
      if (inputNewFolderName) inputNewFolderName.value = '';
      setTimeout(() => {
        hideCreatePanel();
      }, 1200);

      await loadDesignatedFolder();
      await loadAvailableFolders();
      notifyFolderChanged();
    }
  } catch (err) {
    console.error('Failed to create designated folder:', err);
    showFolderFeedback(`Error: ${err}`, 'error');
  } finally {
    if (btnSubmitCreateFolder) btnSubmitCreateFolder.disabled = false;
  }
}

export async function handleBrowseFolder() {
  const info = await browseForFolder();
  if (info) {
    showFolderFeedback(`Selected folder: ${info.current_name}`, 'success');
    await loadDesignatedFolder();
    await loadAvailableFolders();
    notifyFolderChanged();
  }
}

export async function handleRenameFolder() {
  if (!state.currentDesignatedFolder) return;
  if (state.currentDesignatedFolder.current_path.toLowerCase() === state.currentDesignatedFolder.base_path.toLowerCase()) {
    showRenameFolderFeedback('The default Zentra root folder cannot be renamed.', 'error');
    return;
  }
  const rawName = inputRenameFolderName ? inputRenameFolderName.value.trim() : '';
  if (!rawName) {
    showRenameFolderFeedback('Please enter a new folder name.', 'error');
    if (inputRenameFolderName) inputRenameFolderName.focus();
    return;
  }

  try {
    if (btnSubmitRenameFolder) btnSubmitRenameFolder.disabled = true;
    const info = await renameDesignatedFolder(state.currentDesignatedFolder.current_path, rawName);
    if (info) {
      showRenameFolderFeedback(`Folder renamed to "${info.current_name}"!`, 'success');
      if (inputRenameFolderName) inputRenameFolderName.value = '';
      setTimeout(() => {
        hideRenamePanel();
      }, 1200);

      await loadDesignatedFolder();
      await loadAvailableFolders();
      notifyFolderChanged();
    }
  } catch (err) {
    console.error('Failed to rename folder:', err);
    showRenameFolderFeedback(`Error: ${err}`, 'error');
  } finally {
    if (btnSubmitRenameFolder) btnSubmitRenameFolder.disabled = false;
  }
}

export async function handleDeleteFolder() {
  if (!state.currentDesignatedFolder) return;
  if (state.currentDesignatedFolder.current_path.toLowerCase() === state.currentDesignatedFolder.base_path.toLowerCase()) {
    alert('The default Zentra root folder cannot be deleted.');
    return;
  }

  const name = state.currentDesignatedFolder.current_name;
  const path = state.currentDesignatedFolder.current_path;

  if (!confirm(`Are you sure you want to permanently delete folder "${name}" and all recorded sessions inside it from disk?`)) {
    return;
  }

  try {
    if (btnDeleteFolder) btnDeleteFolder.disabled = true;
    const info = await deleteDesignatedFolder(path);
    if (info) {
      showFolderFeedback(`Folder "${name}" deleted. Switched to ${info.current_name}.`, 'success');
      hideAllPanels();

      await loadDesignatedFolder();
      await loadAvailableFolders();
      notifyFolderChanged();
    }
  } catch (err) {
    console.error('Failed to delete designated folder:', err);
    alert(`Could not delete folder: ${err}`);
  } finally {
    if (btnDeleteFolder) btnDeleteFolder.disabled = false;
  }
}

export function initFolderController() {
  initPanels();
  initFolderView();

  const btnToggleCreateFolder = document.getElementById('btn-toggle-create-folder');
  const btnToggleRenameFolder = document.getElementById('btn-toggle-rename-folder');
  btnDeleteFolder = document.getElementById('btn-delete-folder');
  const btnBrowseFolder = document.getElementById('btn-browse-folder');
  const btnOpenActiveFolder = document.getElementById('btn-open-active-folder');
  selectDesignatedFolder = document.getElementById('select-designated-folder');
  inputNewFolderName = document.getElementById('input-new-folder-name');
  btnSubmitCreateFolder = document.getElementById('btn-submit-create-folder');
  const btnCancelCreateFolder = document.getElementById('btn-cancel-create-folder');
  inputRenameFolderName = document.getElementById('input-rename-folder-name');
  btnSubmitRenameFolder = document.getElementById('btn-submit-rename-folder');
  const btnCancelRenameFolder = document.getElementById('btn-cancel-rename-folder');
  const btnOpenFolder = document.getElementById('btn-open-folder');

  if (selectDesignatedFolder) {
    selectDesignatedFolder.addEventListener('change', (e) => {
      handleFolderSelectChange(e.target.value);
    });
  }

  if (btnOpenFolder) {
    btnOpenFolder.addEventListener('click', openOutputFolder);
  }

  if (btnOpenActiveFolder) {
    btnOpenActiveFolder.addEventListener('click', openOutputFolder);
  }

  if (btnToggleCreateFolder) {
    btnToggleCreateFolder.addEventListener('click', toggleCreatePanel);
  }

  if (btnCancelCreateFolder) {
    btnCancelCreateFolder.addEventListener('click', hideCreatePanel);
  }

  if (btnSubmitCreateFolder) {
    btnSubmitCreateFolder.addEventListener('click', handleCreateFolder);
  }

  if (inputNewFolderName) {
    inputNewFolderName.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        handleCreateFolder();
      } else if (e.key === 'Escape') {
        hideCreatePanel();
      }
    });
  }

  if (btnBrowseFolder) {
    btnBrowseFolder.addEventListener('click', handleBrowseFolder);
  }

  if (btnToggleRenameFolder) {
    btnToggleRenameFolder.addEventListener('click', () => {
      if (!state.currentDesignatedFolder) return;
      if (state.currentDesignatedFolder.current_path.toLowerCase() === state.currentDesignatedFolder.base_path.toLowerCase()) {
        alert('The default Zentra root folder cannot be renamed. Create or choose a custom folder to rename.');
        return;
      }
      toggleRenamePanel(state.currentDesignatedFolder.current_name);
    });
  }

  if (btnCancelRenameFolder) {
    btnCancelRenameFolder.addEventListener('click', hideRenamePanel);
  }

  if (btnSubmitRenameFolder) {
    btnSubmitRenameFolder.addEventListener('click', handleRenameFolder);
  }

  if (inputRenameFolderName) {
    inputRenameFolderName.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        handleRenameFolder();
      } else if (e.key === 'Escape') {
        hideRenamePanel();
      }
    });
  }

  if (btnDeleteFolder) {
    btnDeleteFolder.addEventListener('click', handleDeleteFolder);
  }
}