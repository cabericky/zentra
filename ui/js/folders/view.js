/**
 * Zentra Folder Management UI View
 * Manages DOM element caching and folder presentation rendering.
 */

let activeFolderBadge = null;
let activeFolderPath = null;
let folderPathDisplay = null;
let folderStatusIndicator = null;
let btnDeleteFolder = null;
let btnToggleRenameFolder = null;
let btnOpenFolder = null;
let selectDesignatedFolder = null;

export function initFolderView() {
  activeFolderBadge = document.getElementById('active-folder-badge');
  activeFolderPath = document.getElementById('active-folder-path');
  folderPathDisplay = document.getElementById('folder-path-display');
  folderStatusIndicator = document.getElementById('folder-status-indicator');
  btnDeleteFolder = document.getElementById('btn-delete-folder');
  btnToggleRenameFolder = document.getElementById('btn-toggle-rename-folder');
  btnOpenFolder = document.getElementById('btn-open-folder');
  selectDesignatedFolder = document.getElementById('select-designated-folder');
}

/**
 * Updates UI displays with designated folder information.
 */
export function renderDesignatedFolderInfo(info) {
  if (!info) return;

  if (activeFolderBadge) {
    activeFolderBadge.textContent = info.current_name;
  }
  if (activeFolderPath) {
    activeFolderPath.textContent = info.current_path;
    activeFolderPath.title = info.current_path;
  }
  if (folderPathDisplay) {
    folderPathDisplay.textContent = info.current_name;
  }
  if (btnOpenFolder) {
    btnOpenFolder.title = info.current_path;
  }
  if (folderStatusIndicator) {
    folderStatusIndicator.className = info.exists
      ? 'folder-status-indicator ready'
      : 'folder-status-indicator error';
    folderStatusIndicator.title = info.exists
      ? 'Folder exists and is ready'
      : 'Folder does not exist on disk!';
  }

  const isDefaultBase = info.current_path.toLowerCase() === info.base_path.toLowerCase();
  if (btnDeleteFolder) {
    btnDeleteFolder.disabled = isDefaultBase;
    btnDeleteFolder.title = isDefaultBase
      ? 'The default root folder cannot be deleted'
      : 'Delete this designated folder and its sessions';
  }
  if (btnToggleRenameFolder) {
    btnToggleRenameFolder.disabled = isDefaultBase;
    btnToggleRenameFolder.title = isDefaultBase
      ? 'The default root folder cannot be renamed'
      : 'Rename the current designated folder';
  }
}

/**
 * Populates the designated folder selector dropdown.
 */
export function renderFolderDropdown(availableFolders) {
  if (!selectDesignatedFolder || !availableFolders) return;

  selectDesignatedFolder.innerHTML = '';
  availableFolders.forEach((f) => {
    const opt = document.createElement('option');
    opt.value = f.path;
    const statusSuffix = f.exists ? '' : ' [Missing]';
    opt.textContent = `${f.name} (${f.session_count} video${f.session_count === 1 ? '' : 's'})${statusSuffix}`;
    if (f.is_active) {
      opt.selected = true;
    }
    if (!f.exists) {
      opt.disabled = true;
    }
    selectDesignatedFolder.appendChild(opt);
  });
}