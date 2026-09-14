/**
 * Zentra Feature Event Bus
 * Decouples cross-feature communication and reactivity using standard domain events.
 */

class EventEmitter {
  constructor() {
    this.target = new EventTarget();
    this.listenerMap = new Map();
  }

  /**
   * Subscribes a listener to a domain event.
   * Returns an unsubscribe function.
   */
  on(event, handler) {
    const wrapped = (e) => handler(e.detail);
    if (!this.listenerMap.has(handler)) {
      this.listenerMap.set(handler, new Map());
    }
    this.listenerMap.get(handler).set(event, wrapped);

    this.target.addEventListener(event, wrapped);
    return () => this.off(event, handler);
  }

  /**
   * Unsubscribes a listener.
   */
  off(event, handler) {
    const eventMap = this.listenerMap.get(handler);
    if (eventMap && eventMap.has(event)) {
      const wrapped = eventMap.get(event);
      this.target.removeEventListener(event, wrapped);
      eventMap.delete(event);
    }
  }

  /**
   * Emits an event with optional payload data.
   */
  emit(event, detail = {}) {
    this.target.dispatchEvent(new CustomEvent(event, { detail }));
  }
}

export const eventBus = new EventEmitter();

export const Events = {
  SESSION_UPDATED: 'zentra:session-updated',
  FOLDER_CHANGED: 'zentra:folder-changed',
  AVAILABLE_FOLDERS_UPDATED: 'zentra:available-folders-updated',
  EXPORT_COMPLETED: 'zentra:export-completed',
  RECORDING_STATE_CHANGED: 'zentra:recording-state-changed',
  HOTKEYS_UPDATED: 'zentra:hotkeys-updated',
};