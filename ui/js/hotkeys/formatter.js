/**
 * Zentra Hotkey Keycode & Label Formatter
 * Pure utility module mapping Windows virtual-key codes, browser key events, and display strings.
 */

/**
 * Cleanly formats key names for user-facing shortcut badges and descriptions.
 * @param {number} keyCode - Windows Virtual-Key code / e.keyCode
 * @param {string} keyName - Event key property
 * @param {string} code - Event code property
 * @returns {string} Human-friendly display label
 */
export function formatKeyLabel(keyCode, keyName, code) {
  if (keyCode === 192 || code === 'Backquote' || keyName === '`') {
    return '` (Backtick)';
  }
  if (keyCode === 27 || code === 'Escape' || keyName === 'Escape') {
    return 'Esc';
  }
  if (keyCode >= 112 && keyCode <= 123) {
    // F1 - F12
    return `F${keyCode - 111}`;
  }
  if (keyCode === 32 || code === 'Space') {
    return 'Space';
  }
  if (keyCode === 9 || code === 'Tab') {
    return 'Tab';
  }
  if (keyCode === 13 || code === 'Enter') {
    return 'Enter';
  }
  if (keyCode === 45 || code === 'Insert') {
    return 'Insert';
  }
  if (keyCode === 46 || code === 'Delete') {
    return 'Delete';
  }
  if (keyCode === 33 || code === 'PageUp') {
    return 'PageUp';
  }
  if (keyCode === 34 || code === 'PageDown') {
    return 'PageDown';
  }
  if (keyCode === 35 || code === 'End') {
    return 'End';
  }
  if (keyCode === 36 || code === 'Home') {
    return 'Home';
  }
  if (keyCode === 19 || code === 'Pause') {
    return 'Pause';
  }
  if (keyCode >= 65 && keyCode <= 90) {
    // A - Z
    return String.fromCharCode(keyCode);
  }
  if (keyCode >= 48 && keyCode <= 57) {
    // 0 - 9
    return String.fromCharCode(keyCode);
  }

  // Common OEM Punctuation Keys
  if (keyCode === 186 || code === 'Semicolon') return ';';
  if (keyCode === 187 || code === 'Equal') return '=';
  if (keyCode === 188 || code === 'Comma') return ',';
  if (keyCode === 189 || code === 'Minus') return '-';
  if (keyCode === 190 || code === 'Period') return '.';
  if (keyCode === 191 || code === 'Slash') return '/';
  if (keyCode === 219 || code === 'BracketLeft') return '[';
  if (keyCode === 220 || code === 'Backslash') return '\\';
  if (keyCode === 221 || code === 'BracketRight') return ']';
  if (keyCode === 222 || code === 'Quote') return "'";

  return keyName && keyName.length === 1 ? keyName.toUpperCase() : (keyName || `VK_${keyCode}`);
}