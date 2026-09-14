/**
 * Zentra Display Visual Thumbnails Component
 * Responsible for rendering and managing visual display cards in the UI.
 */

export function renderThumbnails(container, monitors, selectedId, onSelect) {
  if (!container) return;
  container.innerHTML = '';

  monitors.forEach((m, idx) => {
    const isSelected = m.id === selectedId;
    const card = document.createElement('div');
    card.className = `monitor-card${isSelected ? ' selected' : ''}`;
    card.setAttribute('data-monitor-id', m.id);
    card.setAttribute('tabindex', '0');
    card.setAttribute('role', 'button');
    card.setAttribute('title', `Click to capture ${m.name}`);

    // Screen preview aspect box with display number
    const screenBox = document.createElement('div');
    screenBox.className = 'monitor-screen-preview';

    const screenIcon = document.createElement('div');
    screenIcon.className = 'monitor-screen-icon';
    screenIcon.innerHTML = '<svg class="icon icon-md"><use href="#icon-monitor"></use></svg>';

    const screenNum = document.createElement('span');
    screenNum.className = 'monitor-screen-num';
    screenNum.textContent = `${idx + 1}`;

    screenBox.appendChild(screenIcon);
    screenBox.appendChild(screenNum);

    // Info metadata
    const infoBox = document.createElement('div');
    infoBox.className = 'monitor-info';

    const titleRow = document.createElement('div');
    titleRow.className = 'monitor-title-row';

    const nameSpan = document.createElement('span');
    nameSpan.className = 'monitor-name';
    nameSpan.textContent = `Display ${idx + 1}`;

    titleRow.appendChild(nameSpan);

    if (m.is_primary) {
      const badge = document.createElement('span');
      badge.className = 'badge badge-primary-monitor';
      badge.textContent = 'Primary';
      titleRow.appendChild(badge);
    }

    const resSpan = document.createElement('span');
    resSpan.className = 'monitor-res';
    resSpan.textContent = `${m.width} × ${m.height}`;

    infoBox.appendChild(titleRow);
    infoBox.appendChild(resSpan);

    card.appendChild(screenBox);
    card.appendChild(infoBox);

    // Interaction handlers
    card.addEventListener('click', () => {
      if (onSelect) onSelect(m.id);
    });

    card.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        if (onSelect) onSelect(m.id);
      }
    });

    container.appendChild(card);
  });
}

export function updateSelectedThumbnail(container, selectedId) {
  if (!container) return;
  const cards = container.querySelectorAll('.monitor-card');
  cards.forEach((card) => {
    if (card.getAttribute('data-monitor-id') === selectedId) {
      card.classList.add('selected');
    } else {
      card.classList.remove('selected');
    }
  });
}

export function setThumbnailsDisabled(container, disabled) {
  if (!container) return;
  const cards = container.querySelectorAll('.monitor-card');
  cards.forEach((c) => {
    if (disabled) {
      c.classList.add('disabled');
      c.setAttribute('tabindex', '-1');
    } else {
      c.classList.remove('disabled');
      c.setAttribute('tabindex', '0');
    }
  });
}