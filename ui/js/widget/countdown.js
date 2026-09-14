/**
 * Zentra Floating Widget Countdown Controller
 * Handles delay presets (Off, 3s, 5s, 10s), interval ticking, beep synchronization, and animation states.
 */

import { playBeep } from './sound.js';
import { formatSeconds } from '../formatters.js';

export const TIMER_OPTIONS = [3, 5, 10, 0]; // 3s, 5s, 10s, 0 (Off)

export class CountdownController {
  constructor({
    timerEl,
    dotEl,
    btnAction,
    btnActionText,
    btnActionIcon,
    btnReturn,
    btnTimer,
    btnTimerText,
    widgetHint,
    emit,
    onComplete,
    onStateChange,
  }) {
    this.timerEl = timerEl;
    this.dotEl = dotEl;
    this.btnAction = btnAction;
    this.btnActionText = btnActionText;
    this.btnActionIcon = btnActionIcon;
    this.btnReturn = btnReturn;
    this.btnTimer = btnTimer;
    this.btnTimerText = btnTimerText;
    this.widgetHint = widgetHint;
    this.emit = emit;
    this.onComplete = onComplete;
    this.onStateChange = onStateChange;

    this.countdownTimerSeconds = 3;
    this.isCountingDown = false;
    this.countdownRemaining = 0;
    this.countdownInterval = null;

    this.init();
  }

  init() {
    this.loadCountdownTimer();

    if (this.btnTimer) {
      this.btnTimer.addEventListener('click', (e) => {
        e.stopPropagation();
        if (this.isCountingDown) return;
        this.cycleTimer();
      });
    }

    this.cancelKeyLabel = 'Esc';
    this.cancelKeyCode = 27;

    // Cancel countdown on Escape key or custom cancel key
    window.addEventListener('keydown', (e) => {
      if (this.isCountingDown) {
        if (e.key === 'Escape' || (this.cancelKeyCode && e.keyCode === this.cancelKeyCode)) {
          e.preventDefault();
          this.cancelCountdown();
        }
      }
    });

    // Cross-window local storage sync
    window.addEventListener('storage', (e) => {
      if (e.key === 'zentra_countdown_timer') {
        this.loadCountdownTimer();
      }
    });
  }

  loadCountdownTimer() {
    const saved = localStorage.getItem('zentra_countdown_timer');
    if (saved !== null) {
      const parsed = parseInt(saved, 10);
      if (TIMER_OPTIONS.includes(parsed)) {
        this.countdownTimerSeconds = parsed;
      }
    }
    this.updateTimerButton();
  }

  updateTimerButton() {
    if (!this.btnTimer || !this.btnTimerText) return;
    if (this.countdownTimerSeconds === 0) {
      this.btnTimerText.textContent = 'Off';
      this.btnTimer.classList.remove('active');
      this.btnTimer.title = 'Countdown Delay: Off (Click to switch: 3s → 5s → 10s → Off)';
    } else {
      this.btnTimerText.textContent = `${this.countdownTimerSeconds}s`;
      this.btnTimer.classList.add('active');
      this.btnTimer.title = `Countdown Delay: ${this.countdownTimerSeconds}s (Click to switch: 3s → 5s → 10s → Off)`;
    }
  }

  setTimerSeconds(seconds) {
    if (typeof seconds === 'number' && TIMER_OPTIONS.includes(seconds)) {
      this.countdownTimerSeconds = seconds;
      this.updateTimerButton();
    }
  }

  cycleTimer() {
    const currentIndex = TIMER_OPTIONS.indexOf(this.countdownTimerSeconds);
    const nextIndex = (currentIndex + 1) % TIMER_OPTIONS.length;
    this.countdownTimerSeconds = TIMER_OPTIONS[nextIndex];
    localStorage.setItem('zentra_countdown_timer', this.countdownTimerSeconds.toString());
    this.updateTimerButton();
    if (this.emit) {
      this.emit('zentra://timer-changed', { seconds: this.countdownTimerSeconds });
    }
  }

  startCountdown() {
    if (this.isCountingDown) return;

    if (this.countdownTimerSeconds <= 0) {
      if (this.onComplete) this.onComplete();
      return;
    }

    this.isCountingDown = true;
    this.countdownRemaining = this.countdownTimerSeconds;

    if (this.dotEl) {
      this.dotEl.classList.remove('recording');
      this.dotEl.classList.add('countdown');
    }
    if (this.timerEl) {
      this.timerEl.classList.add('countdown');
      this.timerEl.textContent = formatSeconds(this.countdownRemaining);
    }

    const cancelHint = this.cancelKeyLabel || 'Esc';
    if (this.btnAction) {
      this.btnAction.className = 'btn-action btn-cancel';
      this.btnAction.title = `Cancel Countdown (${cancelHint})`;
    }
    if (this.btnActionText) this.btnActionText.textContent = 'Cancel';
    if (this.btnActionIcon) this.btnActionIcon.innerHTML = '<use href="#icon-cancel"></use>';

    if (this.btnReturn) this.btnReturn.classList.add('hidden');
    if (this.btnTimer) this.btnTimer.classList.add('hidden');
    if (this.widgetHint) {
      this.widgetHint.textContent = cancelHint;
      this.widgetHint.title = 'Cancel countdown shortcut';
    }

    playBeep(false);

    if (this.onStateChange) this.onStateChange({ isCountingDown: true });

    this.countdownInterval = setInterval(() => {
      this.countdownRemaining -= 1;
      if (this.countdownRemaining > 0) {
        if (this.timerEl) this.timerEl.textContent = formatSeconds(this.countdownRemaining);
        playBeep(false);
      } else {
        clearInterval(this.countdownInterval);
        this.countdownInterval = null;
        this.isCountingDown = false;
        playBeep(true);
        if (this.onComplete) this.onComplete();
      }
    }, 1000);
  }

  cancelCountdown() {
    if (this.countdownInterval) {
      clearInterval(this.countdownInterval);
      this.countdownInterval = null;
    }
    this.isCountingDown = false;
    if (this.onStateChange) this.onStateChange({ isCountingDown: false });
  }

  setCancelKey(label, keyCode) {
    this.cancelKeyLabel = label || 'Esc';
    this.cancelKeyCode = keyCode || 27;
  }
}