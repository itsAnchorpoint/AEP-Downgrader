import type { Ipc } from "./ipc";
import { version } from "../../package.json";
import type {
  AppInfo,
  FileInfo,
  FileKind,
  Job,
  Note,
  OutputOptions,
  Platform,
  Preflight,
  ProgressEvent,
  Problem,
  SystemInfo,
  TargetInfo,
  UpdateInfo,
} from "./ipc-types";
import { outputName } from "./pattern";

const DEMO_PATHS = [
  "/demo/Intro_v05.aep",
  "/demo/Lower_thirds.aep",
  "/demo/Glow_pack.ffx",
  "/demo/._Intro_v05.aep",
];
const MAJORS = [22, 23, 24, 25, 25, 26, 26, 27];
const PLATFORMS: Platform[] = ["windows", "mac-arm", "mac-intel"];
const TARGETS: TargetInfo[] = [26, 25, 24, 23, 22].map((major) => ({
  major,
  name: String(2000 + major),
  status: major >= 24 ? "verified" : "experimental",
}));

function hash(text: string): number {
  let h = 2166136261;
  for (const ch of text) h = Math.imul(h ^ ch.codePointAt(0)!, 16777619);
  return h >>> 0;
}

const baseName = (path: string) => path.split(/[\\/]/).pop() ?? path;

function kindOf(name: string): FileKind {
  if (name.startsWith("._")) return "sidecar";
  switch (name.split(".").pop()?.toLowerCase()) {
    case "aep":
      return "project";
    case "aet":
      return "template";
    case "ffx":
      return "preset";
    case "aepx":
      return "xml";
    default:
      return "unknown";
  }
}

function problemOf(name: string, kind: FileKind): Problem | undefined {
  const lower = name.toLowerCase();
  if (lower.includes("damaged")) return "damaged";
  if (lower.includes("truncated")) return "truncated";
  if (lower.includes("toonew")) return "too-new";
  if (kind === "sidecar") return "sidecar";
  if (kind === "xml") return "xml-project";
  if (kind === "unknown") return "not-ae-file";
  return undefined;
}

function describe(path: string): FileInfo {
  const name = baseName(path);
  const h = hash(name);
  const kind = kindOf(name);
  const problem = problemOf(name, kind);
  const info: FileInfo = { id: String(hash(path)), path, name, size: 40_000 + (h % 60_000_000), kind };
  if (problem) return { ...info, problem };
  const major = MAJORS[h % MAJORS.length];
  return {
    ...info,
    version: {
      major,
      minor: (h >>> 8) % 7,
      patch: 0,
      build: 40 + ((h >>> 16) % 160),
      platform: PLATFORMS[(h >>> 4) % PLATFORMS.length],
      release: major !== 27,
      name: String(2000 + major),
    },
    format: { major: 2, minor: major },
  };
}

function preflightFor(path: string, target: number): Preflight {
  const info = describe(path);
  const year = String(2000 + target);
  const note = (severity: Note["severity"], code: string, params: Note["params"] = {}): Note => ({
    severity,
    code,
    params,
  });
  if (!info.version) return { target, blocked: true, notes: [note("block", "file-unreadable")] };
  if (target >= info.version.major) {
    return { target, blocked: true, notes: [note("block", "target-not-older", { year, from: info.version.name ?? "" })] };
  }

  const notes: Note[] = [];
  {
    if (info.version.major >= 25 && target <= 24) {
      notes.push(note("warn", "shadow-color-removed", { count: 2 }));
      notes.push(note("warn", "effects-removed", { count: 2, names: ["Curl Noise", "Unmult"], year }));
    }
    if (target <= 23) notes.push(note("warn", "text-animator-ranges"));
    if (hash(info.name) % 3 === 0) notes.push(note("info", "plugins-kept", { names: ["Deep Glow", "Trapcode Particular"] }));
    if (notes.length === 0) notes.push(note("info", "no-changes", { year }));
  }
  if (target <= 23) notes.push(note("warn", "experimental-target"));
  return { target, blocked: false, notes };
}

function outputPath(job: Job, options: OutputOptions): string {
  const dir =
    options.location === "next-to-source"
      ? job.path.slice(0, Math.max(job.path.lastIndexOf("/"), 0))
      : options.location.folder;
  return `${dir}/${outputName(options.pattern, baseName(job.path), 2000 + job.target)}`;
}

const MOCK_UPDATE: UpdateInfo = {
  version: "2.1.0",
  notes: "### Added\n\n- Presets (`.ffx`) can be converted.\n\n### Fixed\n\n- Copies keep the colour settings of sources.\n",
  date: "2026-12-01",
  installKind: "installer",
};

/** `stepMs` is how long one file "converts". Tests pass 0 or fake timers. */
export function createMockIpc(stepMs = 700): Ipc {
  const listeners = new Set<(event: ProgressEvent) => void>();
  const emit = (event: ProgressEvent) => listeners.forEach((l) => l(event));
  const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

  async function run(jobs: Job[], options: OutputOptions) {
    for (const job of jobs) {
      emit({ id: job.id, state: "running" });
      await wait(stepMs);
      if (job.path.toLowerCase().includes("fail")) {
        emit({ id: job.id, state: "failed", errorCode: "write-failed" });
      } else {
        emit({ id: job.id, state: "done", outputPath: outputPath(job, options) });
      }
    }
  }

  return {
    inspectPaths: async (paths) => paths.map(describe),
    listTargets: async () => TARGETS,
    preflight: async (path, target) => preflightFor(path, target),
    async convert(jobs, options) {
      jobs.forEach((job) => emit({ id: job.id, state: "queued" }));
      void run(jobs, options);
    },
    revealInFolder: async () => {},
    async onProgress(handler) {
      listeners.add(handler);
      return () => listeners.delete(handler);
    },
    pickFiles: async () => DEMO_PATHS,
    pickFolder: async () => "/demo/Converted",
    async onFileDrop(handlers) {
      // Browsers hide real paths, so dropped files become demo paths by name.
      const over = (e: DragEvent) => {
        e.preventDefault();
        handlers.hover(true);
      };
      const leave = () => handlers.hover(false);
      const drop = (e: DragEvent) => {
        e.preventDefault();
        handlers.hover(false);
        const names = Array.from(e.dataTransfer?.files ?? [], (f) => `/dropped/${f.name}`);
        if (names.length) handlers.drop(names);
      };
      window.addEventListener("dragover", over);
      window.addEventListener("dragleave", leave);
      window.addEventListener("drop", drop);
      return () => {
        window.removeEventListener("dragover", over);
        window.removeEventListener("dragleave", leave);
        window.removeEventListener("drop", drop);
      };
    },
    openUrl: async (url) => {
      window.open(url, "_blank", "noopener,noreferrer");
    },
    systemInfo: async (): Promise<SystemInfo> => {
      const ua = typeof navigator === "undefined" ? "" : navigator.userAgent;
      const os = /Windows/.test(ua) ? "windows" : /Mac/.test(ua) ? "macos" : /Linux/.test(ua) ? "linux" : "unknown";
      return { os, arch: /arm64|aarch64/i.test(ua) ? "aarch64" : /x86_64|x64|Win64|amd64/i.test(ua) ? "x86_64" : "unknown" };
    },
    appInfo: async (): Promise<AppInfo> => ({ version, installKind: "installer", settingsFile: "settings.json" }),
    // Only dev builds pretend that an update exists.
    checkUpdate: async () => {
      await wait(300);
      return import.meta.env.DEV ? MOCK_UPDATE : null;
    },
    async installUpdate(onEvent) {
      const total = 6_000_000;
      onEvent({ event: "started", data: { total } });
      for (let received = 0; received < total; received += 600_000) {
        await wait(stepMs / 5);
        onEvent({ event: "progress", data: { received } });
      }
      onEvent({ event: "finished" });
      await wait(stepMs);
    },
    restartApp: async () => window.location.reload(),
  };
}
