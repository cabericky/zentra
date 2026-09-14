/**
 * Zentra Video Export UI View Component
 * Manages export progress card, fill bar, frame telemetry labels, and reveal actions.
 */

let exportCard = null;
let exportSessionTitle = null;
let exportProgressFill = null;
let exportProgressPercent = null;
let exportProgressFrames = null;
let exportFooterActions = null;
let btnOpenExport = null;

export function initExportUI({ onOpenExportClick }) {
  exportCard = document.getElementById('export-card');
  exportSessionTitle = document.getElementById('export-session-title');
  exportProgressFill = document.getElementById('export-progress-fill');
  exportProgressPercent = document.getElementById('export-progress-percent');
  exportProgressFrames = document.getElementById('export-progress-frames');
  exportFooterActions = document.getElementById('export-footer-actions');
  btnOpenExport = document.getElementById('btn-open-export');

  if (btnOpenExport && onOpenExportClick) {
    btnOpenExport.addEventListener('click', onOpenExportClick);
  }
}

export function isCurrentlyExporting() {
  return exportCard && exportCard.dataset.exporting === 'true';
}

export function setCurrentlyExporting(isExporting) {
  if (exportCard) {
    exportCard.dataset.exporting = isExporting ? 'true' : 'false';
  }
}

export function showExportCard(session) {
  if (!exportCard) return;

  setCurrentlyExporting(true);
  exportCard.style.display = 'flex';

  if (exportSessionTitle) {
    exportSessionTitle.textContent = `Session ${session.id} — ${session.width}x${session.height} @ 60 FPS`;
  }
  if (exportProgressFill) exportProgressFill.style.width = '0%';
  if (exportProgressPercent) exportProgressPercent.textContent = '0%';
  if (exportProgressFrames) {
    exportProgressFrames.textContent = 'Initializing Direct2D compositor & Media Foundation encoder...';
  }
  if (exportFooterActions) exportFooterActions.style.display = 'none';

  exportCard.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
}

export function updateProgress(pct, currentFrame, totalFrames, fps, status) {
  if (exportProgressFill) exportProgressFill.style.width = `${pct}%`;
  if (exportProgressPercent) exportProgressPercent.textContent = `${pct}%`;
  if (exportProgressFrames) {
    exportProgressFrames.textContent = `Frame ${currentFrame} / ${totalFrames} (${fps.toFixed(1)} fps) — ${status}`;
  }

  if (status === 'completed' || pct >= 100) {
    if (exportProgressFill) exportProgressFill.style.width = '100%';
    if (exportProgressPercent) exportProgressPercent.textContent = '100%';
    if (exportFooterActions) exportFooterActions.style.display = 'flex';
  }
}

export function showCompleted(outputPath) {
  if (exportFooterActions) exportFooterActions.style.display = 'flex';
  if (exportProgressFrames) {
    exportProgressFrames.textContent = `Export complete! Saved to ${outputPath}`;
  }
}

export function showFailed(errorMessage) {
  if (exportProgressFrames) {
    exportProgressFrames.textContent = `Export failed: ${errorMessage}`;
  }
}