// Mock only. The real implementation will wrap tauri-plugin-updater once
// signing keys exist; the rest of the app depends on this interface alone.

export interface UpdateInfo {
  version: string;
}

export interface Updater {
  check(): Promise<UpdateInfo | null>;
  install(): Promise<void>;
}

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export function createMockUpdater(available: UpdateInfo | null): Updater {
  return {
    async check() {
      await wait(300);
      return available;
    },
    async install() {
      await wait(1500);
    },
  };
}

// Only dev builds pretend that an update exists.
export const updater: Updater = createMockUpdater(import.meta.env.DEV ? { version: "2.1.0" } : null);
