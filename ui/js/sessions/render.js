/**
 * Zentra Recorded Sessions DOM Renderer
 * Generates session list items, metadata cards, folder pills, and empty state containers.
 */

import { formatDateTime, formatDuration } from '../formatters.js';
import { state } from '../state.js';
import { openSessionFolder, playZoomVideo, renameSession, deleteSession } from './actions.js';

export function renderSessionsList(sessions, {
  sessionsTotalCount,
  sessionsEmpty,
  sessionsList,
  filterSessionFolder,
  onFolderFilterSelect,
  onTriggerExport,
  onReloadSessions,
}) {
  state.allSessions = sessions || [];
  if (sessionsTotalCount) {
    sessionsTotalCount.textContent = state.allSessions.length;
  }

  if (!sessionsEmpty || !sessionsList) return;

  if (state.allSessions.length === 0) {
    sessionsEmpty.style.display = 'block';
    sessionsEmpty.innerHTML = '<p>No recorded sessions yet. Click <strong>Launch Recording Widget</strong> to record your screen.</p>';
    sessionsList.style.display = 'none';
    sessionsList.innerHTML = '';
    return;
  }

  // Filter by folder if a filter is active
  let displayed = state.allSessions;
  if (state.selectedFolderFilter && state.selectedFolderFilter !== '__ALL__') {
    displayed = state.allSessions.filter((s) => {
      return (
        (s.folder_path && s.folder_path.toLowerCase() === state.selectedFolderFilter.toLowerCase()) ||
        (s.folder_name && s.folder_name.toLowerCase() === state.selectedFolderFilter.toLowerCase())
      );
    });
  }

  if (displayed.length === 0) {
    sessionsEmpty.style.display = 'block';
    const activeName = state.currentDesignatedFolder ? state.currentDesignatedFolder.current_name : 'this folder';
    sessionsEmpty.innerHTML = `
      <p>No recorded sessions found in folder <strong>${activeName}</strong>.</p>
      <button id="btn-show-all-folders" class="btn btn-sm btn-secondary" style="margin-top: 10px;">
        View All Folders (${state.allSessions.length})
      </button>
    `;
    const btnShowAll = document.getElementById('btn-show-all-folders');
    if (btnShowAll) {
      btnShowAll.addEventListener('click', () => {
        if (filterSessionFolder) filterSessionFolder.value = '__ALL__';
        state.selectedFolderFilter = '__ALL__';
        if (onFolderFilterSelect) onFolderFilterSelect('__ALL__');
      });
    }
    sessionsList.style.display = 'none';
    sessionsList.innerHTML = '';
    return;
  }

  sessionsEmpty.style.display = 'none';
  sessionsList.style.display = 'block';
  sessionsList.innerHTML = '';

  displayed.forEach((session) => {
    const isExported = session.export_status === 'exported';
    const segName = session.first_segment_name || 'segment_0001.mp4';
    const zoomName = session.export_file_name || `${segName.replace('.mp4', '')}_zoom.mp4`;

    const item = document.createElement('div');
    item.className = 'session-item';

    item.innerHTML = `
      <div class="session-meta">
        <div class="session-title-line">
          <span class="session-title">${segName}</span>
          <span class="session-id-subtext" title="Session timestamp">(${session.id})</span>
          <span class="session-folder-pill" title="Filter by folder: ${session.folder_name} (${session.folder_path})">
            <svg class="icon icon-sm"><use href="#icon-folder"></use></svg> ${session.folder_name || 'Default'}
          </span>
          <span class="tag ${isExported ? 'tag-green' : 'tag-blue'}">
            ${isExported ? 'Zoom Exported' : 'Ready to Export'}
          </span>
        </div>
        <div class="session-submeta">
          <span><svg class="icon icon-sm"><use href="#icon-calendar"></use></svg> ${formatDateTime(session.created_at)}</span>
          <span class="meta-dot">•</span>
          <span><svg class="icon icon-sm"><use href="#icon-clock"></use></svg> ${formatDuration(session.duration_seconds)}</span>
          <span class="meta-dot">•</span>
          <span><svg class="icon icon-sm"><use href="#icon-monitor"></use></svg> ${session.width}×${session.height} @ ${session.fps}fps</span>
          <span class="meta-dot">•</span>
          <span><svg class="icon icon-sm"><use href="#icon-mouse"></use></svg> ${session.click_count} clicks</span>
          ${isExported ? `
            <span class="meta-dot">•</span>
            <span class="meta-export-tag" title="Exported zoom video file"><svg class="icon icon-sm"><use href="#icon-sparkles"></use></svg> <strong>${zoomName}</strong></span>
          ` : ''}
        </div>
      </div>
      <div class="session-actions">
        <button class="btn btn-sm btn-secondary btn-open-session-folder" title="Open directory in Windows Explorer: ${session.output_dir}">
          <svg class="icon icon-sm"><use href="#icon-folder"></use></svg> Folder
        </button>
        ${isExported ? `
          <button class="btn btn-sm btn-play-zoom" title="Open exported zoom video: ${zoomName}">
            <svg class="icon icon-sm"><use href="#icon-play"></use></svg> View Zoom
          </button>
        ` : ''}
        <button class="btn btn-sm ${isExported ? 'btn-exported' : 'btn-export'} btn-export-session">
          <svg class="icon icon-sm"><use href="#icon-sparkles"></use></svg> ${isExported ? 'Re-export Zoom' : 'Export with Zoom'}
        </button>
        <button class="btn btn-sm btn-secondary btn-rename-session" title="Rename this recorded session video file">
          <svg class="icon icon-sm"><use href="#icon-rename"></use></svg> Rename
        </button>
        <button class="btn btn-sm btn-danger btn-delete-session" title="Delete this session and its video file(s)">
          <svg class="icon icon-sm"><use href="#icon-trash"></use></svg> Delete
        </button>
      </div>
    `;

    // Filter by folder on click of folder pill
    const pill = item.querySelector('.session-folder-pill');
    if (pill) {
      pill.addEventListener('click', (e) => {
        e.stopPropagation();
        if (filterSessionFolder) {
          filterSessionFolder.value = session.folder_path;
          state.selectedFolderFilter = session.folder_path;
          if (onFolderFilterSelect) onFolderFilterSelect(session.folder_path);
        }
      });
    }

    // Open directory button
    const btnFolder = item.querySelector('.btn-open-session-folder');
    btnFolder.addEventListener('click', () => {
      openSessionFolder(session.output_dir);
    });

    // View Zoom video button
    const btnPlayZoom = item.querySelector('.btn-play-zoom');
    if (btnPlayZoom) {
      btnPlayZoom.addEventListener('click', () => {
        playZoomVideo(session.output_dir, zoomName);
      });
    }

    // Export button
    const btnExport = item.querySelector('.btn-export-session');
    btnExport.addEventListener('click', () => {
      if (onTriggerExport) onTriggerExport(session);
    });

    // Rename session button
    const btnRenameSession = item.querySelector('.btn-rename-session');
    if (btnRenameSession) {
      btnRenameSession.addEventListener('click', (e) => {
        e.stopPropagation();
        renameSession(session, segName, onReloadSessions);
      });
    }

    // Delete session button
    const btnDeleteSession = item.querySelector('.btn-delete-session');
    if (btnDeleteSession) {
      btnDeleteSession.addEventListener('click', (e) => {
        e.stopPropagation();
        deleteSession(session, segName, onReloadSessions);
      });
    }

    sessionsList.appendChild(item);
  });
}

/**
 * Populates the session folder filter dropdown.
 */
export function renderFolderFilterDropdown(filterEl, availableFolders, allSessionsCount) {
  if (!filterEl) return;

  const currentFilterVal = filterEl.value || state.selectedFolderFilter;
  filterEl.innerHTML = '';

  const allOpt = document.createElement('option');
  allOpt.value = '__ALL__';
  allOpt.textContent = `All Folders (${allSessionsCount || (state.allSessions ? state.allSessions.length : 0)})`;
  filterEl.appendChild(allOpt);

  if (availableFolders && availableFolders.length > 0) {
    availableFolders.forEach((f) => {
      const opt = document.createElement('option');
      opt.value = f.path;
      opt.textContent = `${f.name} (${f.session_count})`;
      filterEl.appendChild(opt);
    });
  }

  const hasOption = Array.from(filterEl.options).some((o) => o.value === currentFilterVal);
  filterEl.value = hasOption ? currentFilterVal : '__ALL__';
  state.selectedFolderFilter = filterEl.value;
}