import { create } from "zustand";
import i18n from "./i18n";
import { ipc } from "./lib/ipc";
import type { FileInfo, OutputOptions, Preflight, ProgressEvent, TargetInfo } from "./lib/ipc-types";
import { defaultSettings, loadSettings, saveSettings, type Settings } from "./lib/settings";
import { updater } from "./lib/updater";

export type Phase = "queue" | "converting" | "done";
export type UpdateStatus = "none" | "available" | "installing" | "installed";

interface AppState {
  settings: Settings;
  view: "main" | "settings";
  phase: Phase;
  files: FileInfo[];
  targets: TargetInfo[];
  target: number | null;
  reports: Record<string, Preflight>;
  expandedId: string | null;
  jobIds: string[];
  progress: Record<string, ProgressEvent>;
  dragging: boolean;
  supportDismissed: boolean;
  whatsNewOpen: boolean;
  update: { status: UpdateStatus; version: string; bannerDismissed: boolean };

  init(): Promise<void>;
  addPaths(paths: string[]): Promise<void>;
  browse(): Promise<void>;
  removeFile(id: string): void;
  clearFiles(): void;
  setTarget(target: number): Promise<void>;
  toggleExpanded(id: string): void;
  convert(): Promise<void>;
  handleProgress(event: ProgressEvent): void;
  reset(): void;
  setDragging(dragging: boolean): void;
  updateSettings(patch: Partial<Settings>): void;
  chooseFolder(): Promise<void>;
  setView(view: "main" | "settings"): void;
  dismissSupport(): void;
  setWhatsNewOpen(open: boolean): void;
  checkUpdate(): Promise<void>;
  installUpdate(): Promise<void>;
  dismissUpdateBanner(): void;
}

const initialState = {
  settings: defaultSettings(),
  view: "main" as const,
  phase: "queue" as Phase,
  files: [] as FileInfo[],
  targets: [] as TargetInfo[],
  target: null as number | null,
  reports: {} as Record<string, Preflight>,
  expandedId: null as string | null,
  jobIds: [] as string[],
  progress: {} as Record<string, ProgressEvent>,
  dragging: false,
  supportDismissed: false,
  whatsNewOpen: false,
  update: { status: "none" as UpdateStatus, version: "", bannerDismissed: false },
};

let initialized = false;
const TERMINAL = new Set<ProgressEvent["state"]>(["done", "failed", "skipped"]);

/** A file can be converted when it has no problem and its report does not block it. */
export function isConvertible(file: FileInfo, reports: Record<string, Preflight>): boolean {
  return !file.problem && !reports[file.id]?.blocked;
}

/** The newest verified target that is older than the newest file; else the first target. */
export function pickTarget(targets: TargetInfo[], files: FileInfo[]): number | null {
  const newest = Math.max(0, ...files.map((f) => f.version?.major ?? 0));
  const best = targets.find((t) => t.status === "verified" && t.major < newest);
  return (best ?? targets[0])?.major ?? null;
}

export function outputOptions(settings: Settings): OutputOptions {
  return {
    location:
      settings.location === "folder" && settings.folder ? { folder: settings.folder } : "next-to-source",
    pattern: settings.pattern,
    onExists: settings.onExists,
  };
}

export const useAppStore = create<AppState>((set, get) => {
  /** Fetches reports for the given files, ignoring the result if the target changed meanwhile. */
  async function loadReports(files: FileInfo[], target: number) {
    const entries = await Promise.all(
      files
        .filter((f) => !f.problem)
        .map(async (f) => [f.id, await ipc.preflight(f.path, target)] as const),
    );
    if (get().target !== target) return;
    set((s) => ({ reports: { ...s.reports, ...Object.fromEntries(entries) } }));
  }

  return {
    ...initialState,

    async init() {
      if (initialized) return; // React StrictMode runs effects twice in dev
      initialized = true;
      const saved = await loadSettings();
      const settings = { ...defaultSettings(), ...saved };
      await i18n.changeLanguage(settings.language);
      set({ settings });
      await ipc.onProgress((event) => get().handleProgress(event));
      set({ targets: await ipc.listTargets() });
      if (settings.autoUpdates) await get().checkUpdate();
    },

    async addPaths(paths) {
      if (get().phase !== "queue" || paths.length === 0) return;
      const known = new Set(get().files.map((f) => f.path));
      const fresh = (await ipc.inspectPaths(paths)).filter((f) => !known.has(f.path));
      if (fresh.length === 0) return;
      const files = [...get().files, ...fresh];
      const target = get().target ?? pickTarget(get().targets, files);
      set({ files, target, view: "main" });
      if (target !== null) await loadReports(fresh, target);
    },

    async browse() {
      await get().addPaths(await ipc.pickFiles());
    },

    removeFile(id) {
      set((s) => ({
        files: s.files.filter((f) => f.id !== id),
        expandedId: s.expandedId === id ? null : s.expandedId,
      }));
    },

    clearFiles() {
      set({ files: [], reports: {}, expandedId: null });
    },

    async setTarget(target) {
      set({ target, reports: {} });
      await loadReports(get().files, target);
    },

    toggleExpanded(id) {
      set((s) => ({ expandedId: s.expandedId === id ? null : id }));
    },

    async convert() {
      const { files, reports, target, settings } = get();
      const ready = files.filter((f) => isConvertible(f, reports));
      if (ready.length === 0 || target === null) return;
      const ids = ready.map((f) => f.id);
      set({
        phase: "converting",
        jobIds: ids,
        expandedId: null,
        progress: Object.fromEntries(ids.map((id) => [id, { id, state: "queued" as const }])),
      });
      try {
        await ipc.convert(
          ready.map((f) => ({ id: f.id, path: f.path, target })),
          outputOptions(settings),
        );
      } catch {
        // The backend refused the whole batch: show every file as failed.
        ids.forEach((id) => get().handleProgress({ id, state: "failed", errorCode: "unknown" }));
      }
    },

    handleProgress(event) {
      const { jobIds } = get();
      if (!jobIds.includes(event.id)) return;
      const progress = { ...get().progress, [event.id]: event };
      const finished = jobIds.every((id) => TERMINAL.has(progress[id]?.state));
      set({ progress, phase: finished ? "done" : "converting" });
    },

    reset() {
      const { settings, targets, update, supportDismissed } = get();
      set({ ...initialState, settings, targets, update, supportDismissed });
    },

    setDragging: (dragging) => set({ dragging }),

    updateSettings(patch) {
      const settings = { ...get().settings, ...patch };
      set({ settings });
      if (patch.language) void i18n.changeLanguage(patch.language);
      void saveSettings(settings);
    },

    async chooseFolder() {
      const folder = await ipc.pickFolder();
      if (folder) get().updateSettings({ folder, location: "folder" });
    },

    setView: (view) => set({ view }),
    dismissSupport: () => set({ supportDismissed: true }),
    setWhatsNewOpen: (open) => set({ whatsNewOpen: open }),

    async checkUpdate() {
      const found = await updater.check().catch(() => null);
      if (found) set({ update: { status: "available", version: found.version, bannerDismissed: false } });
    },

    async installUpdate() {
      set((s) => ({ update: { ...s.update, status: "installing" } }));
      try {
        await updater.install();
        set((s) => ({ update: { ...s.update, status: "installed" } }));
      } catch {
        set((s) => ({ update: { ...s.update, status: "available" } }));
      }
    },

    dismissUpdateBanner: () => set((s) => ({ update: { ...s.update, bannerDismissed: true } })),
  };
});

export { initialState };
