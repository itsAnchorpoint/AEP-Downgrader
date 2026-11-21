import { create } from "zustand";
import i18n from "./i18n";
import { appendEntry, entryFor, type LogEntry, type LogLevel, type LogParams } from "./lib/activity-log";
import { aeMajorForFormat, formatBytes, versionNumber } from "./lib/format";
import { ipc } from "./lib/ipc";
import { compareVersions, hasChangesSince } from "./lib/changelog";
import type { AppInfo, FileInfo, InstallKind, OutputOptions, Preflight, ProgressEvent, TargetInfo } from "./lib/ipc-types";
import { releaseUrl } from "./lib/links";
import {
  defaultSettings,
  loadMemory,
  loadSettings,
  saveMemory,
  saveSettings,
  setSettingsFile,
  type Settings,
} from "./lib/settings";
import changelog from "./content/whatsnew.en.md?raw";

export type Phase = "queue" | "converting" | "done";
export type UpdateStatus = "none" | "available" | "downloading" | "installing" | "installed" | "failed";

export interface UpdateState {
  status: UpdateStatus;
  version: string;
  /** Release notes (Markdown) from the update feed. */
  notes: string | null;
  date: string | null;
  installKind: InstallKind;
  /** Download progress 0..1, or null while the size is unknown. */
  progress: number | null;
  bannerDismissed: boolean;
}

/** "update": notes of the available update. "history": the bundled notes (content/whatsnew.*.md). */
export type WhatsNewView = "update" | "history";

const DAY_MS = 24 * 60 * 60 * 1000;

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
  /** Stop was pressed; the batch is finishing the files that are converting. */
  cancelling: boolean;
  whatsNewView: WhatsNewView;
  /** In the history view, show only versions newer than this (after an update); null shows all. */
  whatsNewSince: string | null;
  appInfo: AppInfo | null;
  update: UpdateState;
  /** A check started from Settings is running. */
  updateChecking: boolean;
  /** Result of the last check started from Settings, for the line under the button. */
  updateCheckResult: "up-to-date" | "found" | "failed" | null;
  /** What the app did in this session, oldest first, capped at LOG_CAP. Never leaves memory except via the copied report. */
  log: LogEntry[];
  logOpen: boolean;
  /** The entry the log panel should scroll to and highlight, set by a row's "Details" button. */
  logFocusId: number | null;
  /** Bumped on every "Details" click so the panel scrolls again to the same entry. */
  logFocusTick: number;
  /** Put full paths into the copied report. Never saved: off on every launch. */
  includePaths: boolean;

  init(): Promise<void>;
  addPaths(paths: string[]): Promise<void>;
  browse(): Promise<void>;
  removeFile(id: string): void;
  clearFiles(): void;
  setTarget(target: number): Promise<void>;
  toggleExpanded(id: string): void;
  convert(): Promise<void>;
  /** Stop after the files converting now. */
  cancelConvert(): Promise<void>;
  handleProgress(event: ProgressEvent): void;
  reset(): void;
  /** After a run: back to the queue with the same files, to convert them to another version. */
  backToQueue(): void;
  setDragging(dragging: boolean): void;
  updateSettings(patch: Partial<Settings>): void;
  chooseFolder(): Promise<void>;
  setView(view: "main" | "settings"): void;
  dismissSupport(): void;
  setWhatsNewOpen(open: boolean, view?: WhatsNewView): void;
  /** `manual`: started by the user from Settings, so the result is shown there. */
  checkUpdate(manual?: boolean): Promise<void>;
  installUpdate(): Promise<void>;
  restartApp(): Promise<void>;
  dismissUpdateBanner(): void;
  setLogOpen(open: boolean): void;
  showLogFor(ref: string): void;
  setIncludePaths(include: boolean): void;
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
  cancelling: false,
  whatsNewView: "history" as WhatsNewView,
  whatsNewSince: null as string | null,
  appInfo: null as AppInfo | null,
  update: {
    status: "none",
    version: "",
    notes: null,
    date: null,
    installKind: "installer",
    progress: null,
    bannerDismissed: false,
  } as UpdateState,
  updateChecking: false,
  updateCheckResult: null as "up-to-date" | "found" | "failed" | null,
  log: [] as LogEntry[],
  logOpen: false,
  logFocusId: null as number | null,
  logFocusTick: 0,
  includePaths: false,
};

let initialized = false;
let nextLogId = 1;
const TERMINAL = new Set<ProgressEvent["state"]>(["done", "failed", "skipped", "cancelled"]);

/** A file can be converted when it has no problem and its report does not block it. */
export function isConvertible(file: FileInfo, reports: Record<string, Preflight>): boolean {
  return !file.problem && !reports[file.id]?.blocked;
}

/** The newest verified target that is older than the newest file; else the first target. */
export function pickTarget(targets: TargetInfo[], files: FileInfo[]): number | null {
  const newest = Math.max(0, ...files.map((f) => f.version?.major ?? aeMajorForFormat(f.format?.major) ?? 0));
  const best = targets.find((t) => t.status === "verified" && t.major < newest);
  return (best ?? targets[0])?.major ?? null;
}

const errorText = (error: unknown) => (error instanceof Error ? error.message : String(error)).slice(0, 300);
const NOTHING_TO_DO = ["not-a-downgrade", "target-not-older"];
const nothingToDo = (report?: Preflight) => report?.notes.some((n) => NOTHING_TO_DO.includes(n.code)) ?? false;
/** Problems that are the file's own damage rather than "not what this app converts". */
const BROKEN: FileInfo["problem"][] = ["truncated", "damaged", "too-new", "unreadable"];

/** `name=value` technical text for the log: what was detected in a file. */
function fileDetail(file: FileInfo): string {
  const parts: string[] = [`kind=${file.kind}`];
  if (file.version) {
    const v = file.version;
    parts.push(`ae=${versionNumber(v)}${v.release ? "" : "-beta"}`, `build=${v.build}`, `platform=${v.platform}`);
  }
  if (file.format) parts.push(`format=${file.format.major}.${file.format.minor}`);
  if (file.problem) parts.push(`problem=${file.problem}`);
  parts.push(`size=${formatBytes(file.size)}`);
  return parts.join(" ");
}

/** `notes=shadow-color-removed:2,plugins-kept:1`: every note code with its count. */
function notesDetail(report: Preflight): string {
  const list = report.notes.map((n) => `${n.code}:${typeof n.params.count === "number" ? n.params.count : 1}`);
  return `target=${report.target} notes=${list.join(",") || "none"}`;
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
    const checked = await Promise.all(
      files
        .filter((f) => !f.problem)
        .map(async (f) => {
          try {
            return { file: f, report: await ipc.preflight(f.path, target) };
          } catch (error) {
            return { file: f, error: errorText(error) };
          }
        }),
    );
    if (get().target !== target) return;
    const year = yearOf(target);
    for (const { file, report, error } of checked) {
      const params = { file: file.path, year };
      if (!report) record("error", "reportFailed", params, error, file.id);
      else if (nothingToDo(report)) record("info", "reportNothingToDo", params, notesDetail(report), file.id);
      else if (report.blocked) record("warning", "reportBlocked", params, notesDetail(report), file.id);
      else if (report.notes.length === 0) record("info", "reportClean", params, notesDetail(report), file.id);
      else {
        const level = report.notes.some((n) => n.severity === "warn") ? "warning" : "info";
        record(level, "reportNotes", { ...params, count: report.notes.length }, notesDetail(report), file.id);
      }
    }
    const reports = Object.fromEntries(checked.flatMap((c) => (c.report ? [[c.file.id, c.report] as const] : [])));
    set((s) => ({ reports: { ...s.reports, ...reports } }));
  }

  /** Adds one entry to the activity log. `coalesce` replaces a directly preceding entry of the same key. */
  function record(
    level: LogLevel,
    key: string,
    params: LogParams = {},
    detail?: string,
    ref?: string,
    coalesce = false,
  ) {
    const entry: LogEntry = { id: nextLogId++, time: Date.now(), level, key, params, detail, ref };
    set((s) => {
      const base = coalesce && s.log.at(-1)?.key === key ? s.log.slice(0, -1) : s.log;
      return { log: appendEntry(base, entry) };
    });
  }
  const yearOf = (target: number | null) => get().targets.find((x) => x.major === target)?.name ?? String(target);

  return {
    ...initialState,

    async init() {
      if (initialized) return; // React StrictMode runs effects twice in dev
      initialized = true;
      const appInfo = await ipc.appInfo();
      setSettingsFile(appInfo.settingsFile);
      const saved = await loadSettings();
      const settings = { ...defaultSettings(), ...saved };
      await i18n.changeLanguage(settings.language);
      set({ settings, appInfo });
      await ipc.onProgress((event) => get().handleProgress(event));
      set({ targets: await ipc.listTargets() });

      // First launch after an update: show what changed since the version that ran before.
      const memory = await loadMemory();
      const before = memory.lastSeenVersion;
      if (before && compareVersions(appInfo.version, before) > 0 && hasChangesSince(changelog, before)) {
        set({ whatsNewOpen: true, whatsNewView: "history", whatsNewSince: before });
      }
      if (before !== appInfo.version) await saveMemory({ ...memory, lastSeenVersion: appInfo.version });

      // At most one automatic check a day: at launch, and again if the app stays open.
      const due = () => get().settings.autoUpdates && get().update.status === "none";
      if (due() && Date.now() - (memory.lastUpdateCheck ?? 0) >= DAY_MS) await get().checkUpdate();
      setInterval(() => {
        if (due()) void get().checkUpdate();
      }, DAY_MS);
    },

    async addPaths(paths) {
      if (get().phase !== "queue" || paths.length === 0) return;
      const known = new Set(get().files.map((f) => f.path));
      let fresh: FileInfo[];
      try {
        fresh = (await ipc.inspectPaths(paths)).filter((f) => !known.has(f.path));
      } catch (error) {
        record("error", "inspectFailed", { count: paths.length }, errorText(error));
        return;
      }
      if (fresh.length === 0) return;
      for (const f of fresh) {
        if (f.problem) {
          const level = BROKEN.includes(f.problem) ? "error" : "warning";
          record(level, "fileRejected", { file: f.path, problem: f.problem }, fileDetail(f), f.id);
        } else {
          record("info", "fileAdded", { file: f.path }, fileDetail(f), f.id);
        }
      }
      const files = [...get().files, ...fresh];
      const chosen = get().target;
      const target = chosen ?? pickTarget(get().targets, files);
      if (chosen === null && target !== null) {
        record("info", "targetChosen", { year: yearOf(target) }, `target=${target} source=auto`);
      }
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
      record("info", "targetChosen", { year: yearOf(target) }, `target=${target} source=user`);
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
      for (const f of files.filter((x) => !ready.includes(x))) {
        if (!f.problem && nothingToDo(reports[f.id])) record("info", "jobNothingToDo", { file: f.path }, undefined, f.id);
        else {
          const why = f.problem ? `problem=${f.problem}` : reports[f.id] ? notesDetail(reports[f.id]) : "no report";
          record("warning", "jobNotConverted", { file: f.path }, why, f.id);
        }
      }
      record(
        "info",
        "convertStarted",
        { count: ready.length, year: yearOf(target) },
        `target=${target} pattern=${settings.pattern} on-exists=${settings.onExists} location=${
          settings.location === "folder" && settings.folder ? "folder" : "next-to-source"
        }`,
      );
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
      } catch (error) {
        // The backend refused the whole batch: show every file as failed.
        record("error", "convertRefused", {}, errorText(error));
        ids.forEach((id) => get().handleProgress({ id, state: "failed", errorCode: "unknown" }));
      }
    },

    handleProgress(event) {
      const { jobIds } = get();
      if (!jobIds.includes(event.id)) return;
      const before = get().progress[event.id]?.state;
      const progress = { ...get().progress, [event.id]: event };
      const finished = jobIds.every((id) => TERMINAL.has(progress[id]?.state));
      const file = get().files.find((f) => f.id === event.id);
      if (file && TERMINAL.has(event.state) && before !== event.state) {
        if (event.state === "done") {
          record("info", "jobDone", { file: file.path, output: event.outputPath ?? "" }, undefined, file.id);
        } else if (event.state === "failed") {
          const code = event.errorCode ?? "unknown";
          record("error", "jobFailed", { file: file.path, error: code }, `code=${code}`, file.id);
        } else if (event.state === "cancelled") {
          record("info", "jobCancelled", { file: file.path }, undefined, file.id);
        } else {
          record("warning", "jobSkipped", { file: file.path }, undefined, file.id);
        }
      }
      if (finished && get().phase !== "done") {
        const states = jobIds.map((id) => progress[id]?.state);
        const count = (state: string) => states.filter((x) => x === state).length;
        record("info", "batchFinished", {
          done: count("done"),
          failed: count("failed"),
          skipped: count("skipped") + count("cancelled"),
        });
      }
      set({ progress, phase: finished ? "done" : "converting", ...(finished ? { cancelling: false } : {}) });
    },

    reset() {
      const { phase, files, target, reports, expandedId, jobIds, progress } = initialState;
      // Only the batch starts over. The activity log, settings and update state belong to the session.
      set({ phase, files, target, reports, expandedId, jobIds, progress, view: "main", dragging: false });
    },

    async cancelConvert() {
      if (get().phase !== "converting" || get().cancelling) return;
      set({ cancelling: true });
      record("info", "batchCancelled", {});
      await ipc.cancelConversion();
    },

    backToQueue() {
      const { jobIds, progress, expandedId } = initialState;
      set({ phase: "queue", jobIds, progress, expandedId, view: "main" });
    },

    setDragging: (dragging) => set({ dragging }),

    updateSettings(patch) {
      const before = get().settings;
      const settings = { ...before, ...patch };
      set({ settings });
      if (patch.folder !== undefined && patch.folder !== before.folder && patch.folder) {
        record("info", "settingFolder", { folder: patch.folder });
      } else if (patch.location && patch.location !== before.location) {
        record("info", "settingLocation", { mode: patch.location });
      }
      if (patch.pattern !== undefined && patch.pattern !== before.pattern) {
        // Typing changes it on every key: keep one entry.
        record("info", "settingPattern", { pattern: patch.pattern }, undefined, undefined, true);
      }
      if (patch.onExists && patch.onExists !== before.onExists) {
        record("info", "settingOnExists", { mode: patch.onExists });
      }
      if (patch.language) void i18n.changeLanguage(patch.language);
      void saveSettings(settings);
    },

    async chooseFolder() {
      const folder = await ipc.pickFolder();
      if (folder) get().updateSettings({ folder, location: "folder" });
    },

    setView: (view) => set({ view }),
    dismissSupport: () => set({ supportDismissed: true }),
    setWhatsNewOpen(open, view) {
      if (!open) return set({ whatsNewOpen: false });
      const hasUpdate = get().update.status !== "none";
      set({ whatsNewOpen: true, whatsNewView: view ?? (hasUpdate ? "update" : "history"), whatsNewSince: null });
    },

    async checkUpdate(manual = false) {
      const busy = ["downloading", "installing", "installed"].includes(get().update.status);
      if (busy || get().updateChecking) return;
      set({ updateChecking: true, ...(manual ? { updateCheckResult: null } : {}) });
      try {
        const found = await ipc.checkUpdate();
        void loadMemory().then((memory) => saveMemory({ ...memory, lastUpdateCheck: Date.now() }));
        if (found) {
          set({
            update: {
              status: "available",
              version: found.version,
              notes: found.notes,
              date: found.date,
              installKind: found.installKind,
              progress: null,
              bannerDismissed: false,
            },
          });
          record("info", "updateAvailable", { version: found.version }, `install=${found.installKind}`);
        } else {
          record("info", "updateUpToDate");
        }
        if (manual) set({ updateCheckResult: found ? "found" : "up-to-date" });
      } catch (error) {
        record("warning", "updateCheckFailed", {}, errorText(error));
        if (manual) set({ updateCheckResult: "failed" });
      } finally {
        set({ updateChecking: false });
      }
    },

    async installUpdate() {
      const { status, version, installKind } = get().update;
      // Installs the app cannot update itself, and failed installs, get the release page.
      if (installKind === "manual" || status === "failed") {
        await ipc.openUrl(releaseUrl(version));
        return;
      }
      if (status !== "available") return;
      set((s) => ({ update: { ...s.update, status: "downloading", progress: null } }));
      let total: number | null = null;
      try {
        await ipc.installUpdate((event) => {
          if (event.event === "started") total = event.data.total;
          else if (event.event === "progress" && total) {
            const progress = Math.min(1, event.data.received / total);
            set((s) => ({ update: { ...s.update, progress } }));
          } else if (event.event === "finished") {
            set((s) => ({ update: { ...s.update, status: "installing", progress: 1 } }));
          }
        });
        set((s) => ({ update: { ...s.update, status: "installed", bannerDismissed: false } }));
        record("info", "updateInstalled", { version });
      } catch (error) {
        set((s) => ({ update: { ...s.update, status: "failed", bannerDismissed: false } }));
        record("error", "updateInstallFailed", { version }, errorText(error));
      }
    },

    async restartApp() {
      await ipc.restartApp();
    },

    dismissUpdateBanner: () => set((s) => ({ update: { ...s.update, bannerDismissed: true } })),

    setLogOpen: (open) => set({ logOpen: open, ...(open ? {} : { logFocusId: null }) }),
    showLogFor(ref) {
      set((s) => ({ logOpen: true, logFocusId: entryFor(s.log, ref)?.id ?? null, logFocusTick: s.logFocusTick + 1 }));
    },
    setIncludePaths: (includePaths) => set({ includePaths }),
  };
});

export { initialState };
