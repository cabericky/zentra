/**
 * Zentra Floating Widget Drag Controller
 * Manages strict hold-to-drag mouse tracking and OS-level window positioning.
 */

export function initWidgetDrag(pillEl, invoke) {
  if (!pillEl) return;

  let isMouseDown = false;
  let initialWinX = 0;
  let initialWinY = 0;
  let startX = 0;
  let startY = 0;

  async function refreshWidgetPosition() {
    try {
      const pos = await invoke('get_widget_position');
      if (pos && Array.isArray(pos)) {
        initialWinX = pos[0];
        initialWinY = pos[1];
      }
    } catch (_) {}
  }

  refreshWidgetPosition();

  pillEl.addEventListener('mousedown', async (e) => {
    // Only drag on primary left click
    if (e.button !== 0) return;
    // Do not drag when clicking interactive buttons
    if (e.target.closest('#btn-action') || e.target.closest('#btn-return') || e.target.closest('#btn-timer')) return;

    isMouseDown = true;
    startX = e.screenX;
    startY = e.screenY;

    // Query current window position
    try {
      const pos = await invoke('get_widget_position');
      if (pos && Array.isArray(pos)) {
        initialWinX = pos[0];
        initialWinY = pos[1];
      }
    } catch (_) {}

    // Trigger OS-level drag
    invoke('start_widget_drag').catch(() => {});
  });

  window.addEventListener('mousemove', (e) => {
    // Strict check: if left button is not pressed, immediately stop
    if ((e.buttons & 1) === 0) {
      isMouseDown = false;
      return;
    }
    if (!isMouseDown) return;

    const dx = e.screenX - startX;
    const dy = e.screenY - startY;
    if (dx !== 0 || dy !== 0) {
      invoke('set_widget_position', {
        x: Math.round(initialWinX + dx),
        y: Math.round(initialWinY + dy),
      }).catch(() => {});
    }
  });

  const handleDragRelease = () => {
    if (isMouseDown) {
      isMouseDown = false;
      refreshWidgetPosition();
    }
  };

  window.addEventListener('mouseup', handleDragRelease);
  document.addEventListener('mouseup', handleDragRelease);
  window.addEventListener('blur', handleDragRelease);
  document.addEventListener('mouseleave', handleDragRelease);
}