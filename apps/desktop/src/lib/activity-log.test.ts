import { describe, expect, it } from "vitest";
import i18n from "../i18n";
import { appendEntry, buildReport, entryFor, hasErrors, LOG_CAP, renderMessage, type LogEntry } from "./activity-log";

const entry = (over: Partial<LogEntry> & Pick<LogEntry, "key">): LogEntry => ({
  id: 1,
  time: Date.UTC(2026, 10, 16, 17, 5, 9),
  level: "info",
  params: {},
  ...over,
});
const en = i18n.getFixedT("en");
const ru = i18n.getFixedT("ru");
const base = { version: "2.0.0", os: "linux", arch: "x86_64", language: "ru", theme: "dark", t: en, now: new Date(Date.UTC(2026, 10, 16, 18)) };

const SOURCE = String.raw`C:\Users\Kristina Ivanova\Clients\Acme\Intro v5.aep`;
const entries: LogEntry[] = [
  entry({ id: 1, key: "fileAdded", params: { file: SOURCE }, detail: "kind=project ae=26.2 build=67 platform=windows size=12.4 MB", ref: "a" }),
  entry({ id: 2, key: "settingFolder", params: { folder: "/home/kristina/Out Folder" } }),
  entry({ id: 3, key: "jobDone", params: { file: SOURCE, output: "/home/kristina/Out Folder/Intro v5_AE2024.aep" }, ref: "a" }),
  entry({
    id: 4,
    level: "error",
    key: "jobFailed",
    params: { file: SOURCE, error: "write-failed" },
    detail: String.raw`code=write-failed os="Permission denied at C:\Users\Kristina Ivanova\Clients\Acme\Intro v5_AE2024.aep"`,
    ref: "a",
  }),
];

describe("log list", () => {
  it("keeps the newest LOG_CAP entries", () => {
    let log: LogEntry[] = [];
    for (let i = 1; i <= LOG_CAP + 25; i++) log = appendEntry(log, entry({ id: i, key: "updateUpToDate" }));
    expect(log).toHaveLength(LOG_CAP);
    expect(log[0].id).toBe(26);
    expect(log.at(-1)?.id).toBe(LOG_CAP + 25);
  });

  it("finds the entry a Details button should open", () => {
    expect(entryFor(entries, "a")?.id).toBe(4); // the error, not the newest-but-fine entry
    expect(entryFor(entries.slice(0, 3), "a")?.id).toBe(3);
    expect(entryFor(entries, "zzz")).toBeUndefined();
    expect(hasErrors(entries)).toBe(true);
    expect(hasErrors(entries.slice(0, 3))).toBe(false);
  });
});

describe("messages", () => {
  it("render in the language asked for, and the panel shows file names only", () => {
    expect(renderMessage(entries[0], en, "name")).toBe("Added Intro v5.aep");
    expect(renderMessage(entries[0], ru, "name")).toBe("Добавлен Intro v5.aep");
    expect(renderMessage(entries[3], en, "name")).toBe("Intro v5.aep failed: Could not write the copy");
  });

  it("translate setting values and plural forms", () => {
    const loc = entry({ key: "settingOnExists", params: { mode: "overwrite" } });
    expect(renderMessage(loc, en, "name")).toBe("If the name already exists: overwrite");
    expect(renderMessage(loc, ru, "name")).toBe("Если имя занято: перезаписать");
    expect(renderMessage(entry({ key: "convertStarted", params: { count: 2, year: "2024" } }), ru, "name")).toBe(
      "Конвертируем 2 файла в After Effects 2024",
    );
  });

  it("has a message for every key the store writes", () => {
    for (const key of ["fileAdded", "fileRejected", "targetChosen", "reportClean", "reportBlocked", "jobDone", "jobFailed", "batchFinished"]) {
      expect(renderMessage(entry({ key, params: { count: 1 } }), en, "name")).not.toBe(key);
    }
  });
});

describe("report", () => {
  it("is in English whatever the app language, with version, system, language and theme", () => {
    const report = buildReport({ ...base, entries, includePaths: false });
    expect(report).toContain("Version: 2.0.0");
    expect(report).toContain("System: linux x86_64");
    expect(report).toContain("Language: ru");
    expect(report).toContain("Theme: dark");
    expect(report).toContain("[17:05:09Z] INFO  Added <file-1>.aep");
    expect(report).toContain("ERROR <file-1>.aep failed: Could not write the copy");
    expect(report).toContain("code=write-failed");
    expect(report).toContain("size=12.4 MB");
  });

  it("redacts paths by default: nothing about the user or folders survives", () => {
    const report = buildReport({ ...base, entries, includePaths: false });
    for (const leak of ["Kristina", "Ivanova", "Clients", "Acme", "Out Folder", "Users", "/home", "C:\\"]) {
      expect(report.toLowerCase()).not.toContain(leak.toLowerCase());
    }
    // One tag per file: the source in every mention, the output on its own.
    expect(report).toContain("saved as <file-2>.aep");
    expect(report).toContain("Save folder: <folder-1>");
    expect(report.match(/<file-1>\.aep/g)?.length).toBeGreaterThanOrEqual(3);
  });

  it("includes full paths only when asked", () => {
    const report = buildReport({ ...base, entries, includePaths: true });
    expect(report).toContain(SOURCE);
    expect(report).toContain("/home/kristina/Out Folder");
    expect(report).toContain("Paths: full");
  });

  it("says so when nothing has happened", () => {
    expect(buildReport({ ...base, entries: [], includePaths: false })).toContain("(no activity yet)");
  });
});
