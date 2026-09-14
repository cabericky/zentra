/**
 * Zentra Folder Management UI Panels & Inline Feedback
 */

let folderFeedbackMsg = null;
let renameFolderFeedbackMsg = null;
let createFolderPanel = null;
let renameFolderPanel = null;
let inputNewFolderName = null;
let inputRenameFolderName = null;

export function initPanels() {
  folderFeedbackMsg = document.getElementById('folder-feedback-msg');
  renameFolderFeedbackMsg = document.getElementById('rename-folder-feedback-msg');
  createFolderPanel = document.getElementById('create-folder-panel');
  renameFolderPanel = document.getElementById('rename-folder-panel');
  inputNewFolderName = document.getElementById('input-new-folder-name');
  inputRenameFolderName = document.getElementById('input-rename-folder-name');
}

export function showFolderFeedback(msg, type) {
  if (!folderFeedbackMsg) return;
  folderFeedbackMsg.textContent = msg;
  folderFeedbackMsg.className = `folder-feedback-msg ${type}`;
  folderFeedbackMsg.style.display = 'block';
}

export function showRenameFolderFeedback(msg, type) {
  if (!renameFolderFeedbackMsg) return;
  renameFolderFeedbackMsg.textContent = msg;
  renameFolderFeedbackMsg.className = `folder-feedback-msg ${type}`;
  renameFolderFeedbackMsg.style.display = 'block';
}

export function toggleCreatePanel() {
  if (!createFolderPanel) return;
  const isVisible = createFolderPanel.style.display !== 'none';
  createFolderPanel.style.display = isVisible ? 'none' : 'block';
  if (renameFolderPanel) renameFolderPanel.style.display = 'none';
  if (folderFeedbackMsg) folderFeedbackMsg.style.display = 'none';
  if (!isVisible && inputNewFolderName) {
    inputNewFolderName.value = '';
    setTimeout(() => inputNewFolderName.focus(), 50);
  }
}

export function hideCreatePanel() {
  if (createFolderPanel) createFolderPanel.style.display = 'none';
  if (folderFeedbackMsg) folderFeedbackMsg.style.display = 'none';
}

export function toggleRenamePanel(currentName) {
  if (!renameFolderPanel) return;
  const isVisible = renameFolderPanel.style.display !== 'none';
  renameFolderPanel.style.display = isVisible ? 'none' : 'block';
  if (createFolderPanel) createFolderPanel.style.display = 'none';
  if (renameFolderFeedbackMsg) renameFolderFeedbackMsg.style.display = 'none';
  if (!isVisible && inputRenameFolderName) {
    inputRenameFolderName.value = currentName || '';
    setTimeout(() => {
      inputRenameFolderName.focus();
      inputRenameFolderName.select();
    }, 50);
  }
}

export function hideRenamePanel() {
  if (renameFolderPanel) renameFolderPanel.style.display = 'none';
  if (renameFolderFeedbackMsg) renameFolderFeedbackMsg.style.display = 'none';
}

export function hideAllPanels() {
  hideCreatePanel();
  hideRenamePanel();
}