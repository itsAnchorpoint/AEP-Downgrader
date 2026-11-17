export type FileKind = "project" | "template" | "preset" | "xml" | "sidecar" | "unknown";
export type Platform = "windows" | "mac-intel" | "mac-arm" | "unknown";
export type Problem =
  | "not-ae-file"
  | "sidecar"
  | "xml-project"
  | "truncated"
  | "damaged"
  | "too-new"
  | "unreadable";

export interface FileInfo {
  id: string;
  path: string;
  name: string;
  size: number;
  kind: FileKind;
  version?: {
    major: number;
    minor: number;
    patch: number;
    build: number;
    platform: Platform;
    release: boolean;
    name?: string; // e.g. "2026"
  };
  format?: { major: number; minor: number };
  problem?: Problem;
}

export interface TargetInfo {
  major: number;
  name: string;
  status: "verified" | "experimental";
}

export interface Note {
  severity: "info" | "warn" | "block";
  code: string;
  params: Record<string, string | number | string[]>;
}

export interface Preflight {
  target: number;
  blocked: boolean;
  notes: Note[];
}

/** Operating system and CPU architecture, for the diagnostic report. */
export interface SystemInfo {
  os: string;
  arch: string;
}

export interface Job {
  id: string;
  path: string;
  target: number;
}

export interface OutputOptions {
  location: "next-to-source" | { folder: string };
  pattern: string;
  onExists: "rename" | "overwrite" | "skip";
}

export interface ProgressEvent {
  id: string;
  state: "queued" | "running" | "done" | "failed" | "skipped";
  outputPath?: string;
  errorCode?: string;
}

/** How this copy of the app gets updates. "manual": it links to the release page instead. */
export type InstallKind = "installer" | "portable" | "manual";

export interface AppInfo {
  version: string;
  installKind: InstallKind;
  /** Settings store: a bare file name (app data folder) or an absolute path (portable app). */
  settingsFile: string;
}

export interface UpdateInfo {
  version: string;
  /** Release notes (Markdown) from the update feed. */
  notes: string | null;
  /** Release date, YYYY-MM-DD. */
  date: string | null;
  installKind: InstallKind;
}

export type DownloadEvent =
  | { event: "started"; data: { total: number | null } }
  | { event: "progress"; data: { received: number } }
  | { event: "finished" };
