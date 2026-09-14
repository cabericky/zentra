/**
 * Zentra Tauri Updater Plugin Guest Binding
 * Compatible with @tauri-apps/plugin-updater API.
 * Uses window.__TAURI__.core with development mock fallback.
 */

const isTauri = typeof window !== 'undefined' && window.__TAURI__ !== undefined;

function convertToRustHeaders(options) {
  if (options?.headers) {
    options.headers = Array.from(new Headers(options.headers).entries());
  }
}

export class Update {
  constructor(metadata) {
    this.rid = metadata.rid;
    this.available = true;
    this.currentVersion = metadata.currentVersion;
    this.version = metadata.version;
    this.date = metadata.date;
    this.body = metadata.body;
    this.rawJson = metadata.rawJson;
    this.downloadedBytes = undefined;
  }

  async download(onEvent, options) {
    if (!isTauri || !window.__TAURI__.core) {
      console.warn('[Mock Updater] download started');
      if (onEvent) {
        onEvent({ event: 'Started', data: { contentLength: 10485760 } });
        onEvent({ event: 'Progress', data: { chunkLength: 5242880 } });
        onEvent({ event: 'Progress', data: { chunkLength: 5242880 } });
        onEvent({ event: 'Finished' });
      }
      return;
    }

    convertToRustHeaders(options);
    const channel = new window.__TAURI__.core.Channel();
    if (onEvent) {
      channel.onmessage = onEvent;
    }
    const downloadedBytesRid = await window.__TAURI__.core.invoke('plugin:updater|download', {
      onEvent: channel,
      rid: this.rid,
      ...options,
    });
    this.downloadedBytes = new window.__TAURI__.core.Resource(downloadedBytesRid);
  }

  async install(options) {
    if (!isTauri || !window.__TAURI__.core) {
      console.warn('[Mock Updater] install simulated');
      return;
    }
    if (!this.downloadedBytes) {
      throw new Error('Update.install called before Update.download');
    }
    await window.__TAURI__.core.invoke('plugin:updater|install', {
      updateRid: this.rid,
      bytesRid: this.downloadedBytes.rid,
      ...options,
    });
    this.downloadedBytes = undefined;
  }

  async downloadAndInstall(onEvent, options) {
    if (!isTauri || !window.__TAURI__.core) {
      console.warn('[Mock Updater] downloadAndInstall simulated');
      if (onEvent) {
        onEvent({ event: 'Started', data: { contentLength: 20971520 } });
        for (let i = 1; i <= 5; i++) {
          await new Promise((resolve) => setTimeout(resolve, 300));
          onEvent({ event: 'Progress', data: { chunkLength: 4194304 } });
        }
        onEvent({ event: 'Finished' });
      }
      return;
    }

    convertToRustHeaders(options);
    const channel = new window.__TAURI__.core.Channel();
    if (onEvent) {
      channel.onmessage = onEvent;
    }
    await window.__TAURI__.core.invoke('plugin:updater|download_and_install', {
      onEvent: channel,
      rid: this.rid,
      ...options,
    });
  }

  async close() {
    if (this.downloadedBytes && typeof this.downloadedBytes.close === 'function') {
      await this.downloadedBytes.close();
    }
    if (isTauri && window.__TAURI__.core?.Resource) {
      try {
        await window.__TAURI__.core.invoke('plugin:resources|close', { rid: this.rid });
      } catch (_) {}
    }
  }
}

export async function check(options) {
  if (!isTauri || !window.__TAURI__.core) {
    console.warn('[Mock Updater] check for updates (browser mock)');
    return null;
  }

  convertToRustHeaders(options);
  const metadata = await window.__TAURI__.core.invoke('plugin:updater|check', {
    ...options,
  });

  return metadata ? new Update(metadata) : null;
}