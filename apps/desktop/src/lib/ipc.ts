import type {
  AppInfo,
  DownloadEvent,
  FileInfo,
  Job,
  OutputOptions,
  Preflight,
  ProgressEvent,
  SystemInfo,
  TargetInfo,
  UpdateInfo,
} from "./ipc-types";
import { createMockIpc } from "./ipc-mock";
import { createTauriIpc, isTauri } from "./ipc-tauri";

export type Unlisten = () => void;

export interface Ipc {
  inspectPaths(paths: string[]): Promise<FileInfo[]>;
  listTargets(): Promise<TargetInfo[]>;
  preflight(path: string, target: number): Promise<Preflight>;
  convert(jobs: Job[], options: OutputOptions): Promise<void>;
  revealInFolder(path: string): Promise<void>;
  onProgress(handler: (event: ProgressEvent) => void): Promise<Unlisten>;
  pickFiles(): Promise<string[]>;
  pickFolder(): Promise<string | null>;
  onFileDrop(handlers: { hover: (over: boolean) => void; drop: (paths: string[]) => void }): Promise<Unlisten>;
  openUrl(url: string): Promise<void>;
  systemInfo(): Promise<SystemInfo>;
  appInfo(): Promise<AppInfo>;
  /** Asks the update feed for a newer version. The only network request of the app. */
  checkUpdate(): Promise<UpdateInfo | null>;
  /** Downloads, verifies and installs the update found by the last check. */
  installUpdate(onEvent: (event: DownloadEvent) => void): Promise<void>;
  restartApp(): Promise<void>;
}

// The real backend inside Tauri, an in-browser mock everywhere else (pnpm dev, tests).
export const ipc: Ipc = isTauri() ? createTauriIpc() : createMockIpc();
