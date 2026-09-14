/**
 * Zentra Recorded Sessions Service
 * Manages IPC queries for session listing and storage retrieval.
 */

import { invoke } from '../ipc.js';
import { state } from '../state.js';

export async function fetchSessions() {
  try {
    const sessions = await invoke('list_sessions');
    state.allSessions = sessions || [];
    return state.allSessions;
  } catch (err) {
    console.error('Failed to load sessions:', err);
    return [];
  }
}