import { LazyStore } from "@tauri-apps/plugin-store";
import { isTauri } from "@tauri-apps/api/core";
import { DEFAULT_PATTERN } from "./pattern";
import { detectLanguage, type Language } from "./locale";

/** Options for the accent color; the palettes themselves live in styles/accents.css. */
export const ACCENTS = ["lime", "ember", "pink", "aqua", "cobalt"] as const;
export type Accent = (typeof ACCENTS)[number];

/** "system" follows the OS (light or dark); "ae" is a full theme with its own fixed colors. */
export const THEMES = ["system", "light", "dark", "ae"] as const;
export type Theme = (typeof THEMES)[number];

/** Themes that bring their own accent; the accent setting is kept but not applied. */
export const usesOwnAccent = (theme: Theme) => theme === "ae";

export interface Settings {
  language: Language;
  theme: Theme;
  accent: Accent;
  location: "next-to-source" | "folder";
  folder: string | null;
  pattern: string;
  onExists: "rename" | "overwrite" | "skip";
  autoUpdates: boolean;
}

export function defaultSettings(): Settings {
  return {
    language: detectLanguage(),
    theme: "system",
    accent: "lime",
    location: "next-to-source",
    folder: null,
    pattern: DEFAULT_PATTERN,
    onExists: "rename",
    autoUpdates: true,
  };
}

const KEY = "settings";
const MEMORY_KEY = "memory";
let tauriStore: LazyStore | null = null;

/**
 * Where settings are kept, from `appInfo().settingsFile`: the app data folder, or next to the
 * exe for the portable Windows app. Call before the first load.
 */
export function setSettingsFile(file: string) {
  tauriStore = new LazyStore(file);
}

const store = () => (tauriStore ??= new LazyStore("settings.json"));

async function read<T>(key: string): Promise<Partial<T>> {
  try {
    if (isTauri()) return (await store().get<Partial<T>>(key)) ?? {};
    return JSON.parse(localStorage.getItem(key) ?? "{}");
  } catch {
    return {};
  }
}

async function write(key: string, value: unknown): Promise<void> {
  try {
    if (isTauri()) await store().set(key, value);
    else localStorage.setItem(key, JSON.stringify(value));
  } catch (error) {
    console.error(`Could not save ${key}`, error);
  }
}

export const loadSettings = () => read<Settings>(KEY);
export const saveSettings = (settings: Settings) => write(KEY, settings);

/** What the app remembers between launches that is not a setting. */
export interface Memory {
  /** The version that ran last; a newer one opens "What's new" with the changes since. */
  lastSeenVersion: string;
  /** When the last update check succeeded (ms since epoch). */
  lastUpdateCheck: number;
}

export const loadMemory = () => read<Memory>(MEMORY_KEY);
export const saveMemory = (memory: Partial<Memory>) => write(MEMORY_KEY, memory);
