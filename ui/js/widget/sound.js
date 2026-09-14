/**
 * Zentra Widget Audio Synthesizer
 * Generates synthetic acoustic countdown feedback using the Web Audio API with zero external audio assets.
 */

let audioContext = null;

/**
 * Plays a pleasant sine wave beep.
 * @param {boolean} isFinal - If true, plays a higher frequency confirmation tone (880 Hz / A5);
 *                            otherwise plays a standard countdown tick (587.33 Hz / D5).
 */
export function playBeep(isFinal = false) {
  try {
    if (!audioContext) {
      const AudioCtx = window.AudioContext || window.webkitAudioContext;
      if (AudioCtx) audioContext = new AudioCtx();
    }
    if (audioContext && audioContext.state === 'suspended') {
      audioContext.resume();
    }
    if (!audioContext) return;

    const osc = audioContext.createOscillator();
    const gain = audioContext.createGain();
    osc.type = 'sine';
    osc.frequency.setValueAtTime(isFinal ? 880.0 : 587.33, audioContext.currentTime);
    gain.gain.setValueAtTime(0.12, audioContext.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.0001, audioContext.currentTime + (isFinal ? 0.25 : 0.12));
    osc.connect(gain);
    gain.connect(audioContext.destination);
    osc.start();
    osc.stop(audioContext.currentTime + (isFinal ? 0.25 : 0.12));
  } catch (_) {}
}