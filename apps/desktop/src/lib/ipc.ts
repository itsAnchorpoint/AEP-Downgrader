import type { FileInfo, Job, OutputOptions, Preflight, ProgressEvent, TargetInfo } from "./ipc-types";
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
}

// The real backend inside Tauri, an in-browser mock everywhere else (pnpm dev, tests).
export const ipc: Ipc = isTauri() ? createTauriIpc() : createMockIpc();
