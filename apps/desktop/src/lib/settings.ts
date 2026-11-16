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
const tauriStore = new LazyStore("settings.json");

export async function loadSettings(): Promise<Partial<Settings>> {
  try {
    if (isTauri()) return (await tauriStore.get<Partial<Settings>>(KEY)) ?? {};
    return JSON.parse(localStorage.getItem(KEY) ?? "{}");
  } catch {
    return {};
  }
}

export async function saveSettings(settings: Settings): Promise<void> {
  try {
    if (isTauri()) await tauriStore.set(KEY, settings);
    else localStorage.setItem(KEY, JSON.stringify(settings));
  } catch (error) {
    console.error("Could not save settings", error);
  }
}
