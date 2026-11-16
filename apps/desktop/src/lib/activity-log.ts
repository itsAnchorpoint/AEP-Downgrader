import type { TFunction } from "i18next";
import { redactAll } from "./redact";

export type LogLevel = "info" | "warning" | "error";
export type LogParams = Record<string, string | number>;

/** One thing the app did in this session. The text comes from the catalog (`log.msg.<key>`). */
export interface LogEntry {
  id: number;
  /** Milliseconds since the epoch. */
  time: number;
  level: LogLevel;
  key: string;
  params: LogParams;
  /** Technical details, language-independent `name=value` text shown in monospace. */
  detail?: string;
  /** The file this entry is about (its `FileInfo.id`), so a row can jump to it. */
  ref?: string;
}

export const LOG_CAP = 500;

/** Params that hold file system paths. The panel shows their file name; the report redacts them. */
const FILE_PARAMS = ["file", "output"];
const FOLDER_PARAMS = ["folder"];
/** Params that hold a code which is translated where it is shown. */
const CODE_PARAMS: Record<string, string> = { problem: "problem.", error: "error.", mode: "log.value." };

export function appendEntry(log: LogEntry[], entry: LogEntry): LogEntry[] {
  const next = [...log, entry];
  return next.length > LOG_CAP ? next.slice(next.length - LOG_CAP) : next;
}

const baseName = (path: string) => path.split(/[\\/]+/).filter(Boolean).pop() ?? path;

/** The entry's message in the language of `t`. `paths: "name"` shows file names only. */
export function renderMessage(entry: LogEntry, t: TFunction, paths: "name" | "full"): string {
  const params: Record<string, string | number> = {};
  for (const [name, value] of Object.entries(entry.params)) {
    if (name in CODE_PARAMS) {
      params[name] = t(`${CODE_PARAMS[name]}${value}`, { defaultValue: String(value) });
    } else if (paths === "name" && [...FILE_PARAMS, ...FOLDER_PARAMS].includes(name)) {
      params[name] = baseName(String(value));
    } else {
      params[name] = value;
    }
  }
  return t(`log.msg.${entry.key}`, { defaultValue: entry.key, ...params });
}

export const hasErrors = (log: LogEntry[]) => log.some((e) => e.level === "error");

/** The entry a "Details" button should open: the newest problem of that file, else its newest entry. */
export function entryFor(log: LogEntry[], ref: string): LogEntry | undefined {
  const own = log.filter((e) => e.ref === ref);
  return [...own].reverse().find((e) => e.level !== "info") ?? own.at(-1);
}

export interface ReportInput {
  version: string;
  os: string;
  arch: string;
  language: string;
  theme: string;
  entries: LogEntry[];
  /** `t` fixed to English: reports are read by the maintainer. */
  t: TFunction;
  includePaths: boolean;
  now: Date;
}

const clock = (time: number) => new Date(time).toISOString().slice(11, 19) + "Z";
const LEVEL = { info: "INFO ", warning: "WARN ", error: "ERROR" } as const;

/**
 * The plain-text diagnostic report. With `includePaths` off (the default) every path in it is
 * redacted by `redactAll`; nothing else here can contain a path, so that is the single gate.
 */
export function buildReport(input: ReportInput): string {
  const { entries, includePaths, t } = input;
  const messages = entries.map((e) => renderMessage(e, t, "full"));
  const details = entries.map((e) => e.detail ?? "");
  const known = {
    files: entries.flatMap((e) => FILE_PARAMS.map((p) => e.params[p])).filter((v): v is string => typeof v === "string"),
    folders: entries.flatMap((e) => FOLDER_PARAMS.map((p) => e.params[p])).filter((v): v is string => typeof v === "string"),
  };
  // One call over every line so tags are numbered consistently through the whole report.
  const flat = includePaths ? [...messages, ...details] : redactAll([...messages, ...details], known);
  const shownMessages = flat.slice(0, entries.length);
  const shownDetails = flat.slice(entries.length);

  const lines = [
    "AEP Downgrader diagnostic report",
    `Version: ${input.version}`,
    `System: ${input.os} ${input.arch}`,
    `Language: ${input.language}`,
    `Theme: ${input.theme}`,
    `Created: ${input.now.toISOString().slice(0, 19)}Z`,
    includePaths ? "Paths: full (included on request)" : "Paths: redacted (<file-N> tags, extensions kept)",
    `Entries: ${entries.length}${entries.length >= LOG_CAP ? ` (last ${LOG_CAP} kept)` : ""}`,
    "",
  ];
  if (entries.length === 0) lines.push("(no activity yet)");
  entries.forEach((entry, i) => {
    lines.push(`[${clock(entry.time)}] ${LEVEL[entry.level]} ${shownMessages[i]}`);
    if (shownDetails[i]) lines.push(`                      ${shownDetails[i]}`);
  });
  return lines.join("\n") + "\n";
}
