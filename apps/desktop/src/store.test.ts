import { beforeEach, describe, expect, it } from "vitest";
import { ipc } from "./lib/ipc";
import { detectLanguage } from "./lib/locale";
import { initialState, isConvertible, outputOptions, pickTarget, useAppStore } from "./store";
import { defaultSettings } from "./lib/settings";

const store = () => useAppStore.getState();

beforeEach(async () => {
  useAppStore.setState({ ...initialState });
  useAppStore.setState({ targets: await ipc.listTargets() });
});

describe("adding files", () => {
  it("adds files once, picks a default target and loads reports", async () => {
    await store().addPaths(["/a/Intro.aep", "/a/Lower.aep"]);
    await store().addPaths(["/a/Intro.aep"]);
    expect(store().files).toHaveLength(2);
    expect(store().target).not.toBeNull();
    expect(Object.keys(store().reports)).toHaveLength(2);
  });

  it("keeps problem files in the list but never converts them", async () => {
    await store().addPaths(["/a/._Intro.aep"]);
    const [file] = store().files;
    expect(file.problem).toBe("sidecar");
    expect(isConvertible(file, store().reports)).toBe(false);
  });

  it("refreshes reports when the target changes", async () => {
    await store().addPaths(["/a/Intro.aep"]);
    await store().setTarget(22);
    const [file] = store().files;
    expect(store().target).toBe(22);
    expect(store().reports[file.id].target).toBe(22);
  });

  it("removes and clears files", async () => {
    await store().addPaths(["/a/Intro.aep", "/a/Lower.aep"]);
    store().removeFile(store().files[0].id);
    expect(store().files).toHaveLength(1);
    store().clearFiles();
    expect(store().files).toHaveLength(0);
  });
});

describe("converting", () => {
  it("moves to done once every job reached a final state", async () => {
    await store().addPaths(["/a/Intro.aep", "/a/Lower.aep"]);
    await store().setTarget(22);
    const jobs = store().files.filter((f) => isConvertible(f, store().reports));
    await store().convert();
    expect(store().phase).toBe("converting");
    expect(store().jobIds).toEqual(jobs.map((f) => f.id));

    store().handleProgress({ id: jobs[0].id, state: "done", outputPath: "/a/x.aep" });
    expect(store().phase).toBe("converting");
    for (const job of jobs) store().handleProgress({ id: job.id, state: "failed", errorCode: "write-failed" });
    expect(store().phase).toBe("done");
  });

  it("ignores events for unknown ids and does nothing without convertible files", async () => {
    await store().addPaths(["/a/._Intro.aep"]);
    await store().convert();
    expect(store().phase).toBe("queue");
    store().handleProgress({ id: "nope", state: "done" });
    expect(store().progress).toEqual({});
  });

  it("reset returns to an empty queue and keeps settings", async () => {
    await store().addPaths(["/a/Intro.aep"]);
    store().updateSettings({ pattern: "{name}_x" });
    store().reset();
    expect(store().files).toHaveLength(0);
    expect(store().settings.pattern).toBe("{name}_x");
  });
});

describe("accent color", () => {
  it("defaults to lime, can be changed and survives reset", () => {
    expect(store().settings.accent).toBe("lime");
    store().updateSettings({ accent: "pink" });
    store().reset();
    expect(store().settings.accent).toBe("pink");
  });
});

describe("helpers", () => {
  it("picks the newest verified target older than the newest file", () => {
    const targets = [
      { major: 26, name: "2026", status: "verified" as const },
      { major: 25, name: "2025", status: "verified" as const },
      { major: 22, name: "2022", status: "experimental" as const },
    ];
    const file = (major: number) => ({ version: { major } }) as never;
    expect(pickTarget(targets, [file(26)])).toBe(25);
    expect(pickTarget(targets, [file(22)])).toBe(26);
    expect(pickTarget([], [])).toBeNull();
  });

  it("builds output options from settings", () => {
    const s = defaultSettings();
    expect(outputOptions(s).location).toBe("next-to-source");
    expect(outputOptions({ ...s, location: "folder", folder: "/out" }).location).toEqual({ folder: "/out" });
    expect(outputOptions({ ...s, location: "folder", folder: null }).location).toBe("next-to-source");
  });

  it("detects Russian from the OS locale", () => {
    expect(detectLanguage("ru-RU")).toBe("ru");
    expect(detectLanguage("en-US")).toBe("en");
    expect(detectLanguage("de")).toBe("en");
  });
});

describe("activity log", () => {
  const keys = () => store().log.map((e) => e.key);

  it("starts empty with full paths off", () => {
    expect(store().log).toEqual([]);
    expect(store().includePaths).toBe(false);
  });

  it("records added files, the chosen target and each report", async () => {
    await store().addPaths(["/demo/Intro_v05.aep", "/a/._Lower.aep", "/a/Damaged.aep"]);
    expect(keys()).toEqual(expect.arrayContaining(["fileAdded", "fileRejected", "targetChosen"]));
    expect(keys().some((k) => k.startsWith("report"))).toBe(true);
    const sidecar = store().log.find((e) => e.params.file === "/a/._Lower.aep");
    expect(sidecar?.level).toBe("warning");
    expect(store().log.find((e) => e.params.file === "/a/Damaged.aep")?.level).toBe("error");
    const report = store().log.find((e) => e.key.startsWith("report"));
    expect(report?.detail).toMatch(/notes=/);
    expect(store().log.every((e) => e.time > 0 && e.id > 0)).toBe(true);
  });

  it("records a user's target change and settings that affect output", async () => {
    await store().addPaths(["/a/Intro.aep"]);
    await store().setTarget(22);
    store().updateSettings({ onExists: "skip" });
    store().updateSettings({ pattern: "{name}_a" });
    store().updateSettings({ pattern: "{name}_ab" });
    store().updateSettings({ theme: "dark" });
    expect(store().log.filter((e) => e.key === "targetChosen").at(-1)?.detail).toContain("source=user");
    expect(keys()).toContain("settingOnExists");
    // Typing in the pattern field keeps one entry; theme is not an output setting.
    expect(store().log.filter((e) => e.key === "settingPattern")).toHaveLength(1);
    expect(store().log.find((e) => e.key === "settingPattern")?.params.pattern).toBe("{name}_ab");
    expect(keys().some((k) => k.toLowerCase().includes("theme"))).toBe(false);
  });

  it("records every conversion outcome and a summary", async () => {
    await store().addPaths(["/demo/Intro_v05.aep", "/demo/Lower_thirds.aep"]);
    await store().setTarget(22);
    const ready = store().files.filter((f) => isConvertible(f, store().reports));
    expect(ready.length).toBeGreaterThan(1);
    await store().convert();
    ready.forEach((f, i) =>
      store().handleProgress(
        i === 0
          ? { id: f.id, state: "failed", errorCode: "write-failed" }
          : { id: f.id, state: "done", outputPath: "/demo/out.aep" },
      ),
    );
    expect(keys()).toEqual(expect.arrayContaining(["convertStarted", "jobDone", "jobFailed", "batchFinished"]));
    const failed = store().log.find((e) => e.key === "jobFailed");
    expect(failed?.level).toBe("error");
    expect(failed?.ref).toBe(ready[0].id);
    expect(failed?.detail).toBe("code=write-failed");
  });

  it("is kept when the user starts another batch", async () => {
    await store().addPaths(["/a/Intro.aep"]);
    store().setIncludePaths(true);
    const before = store().log.length;
    store().reset();
    expect(store().log).toHaveLength(before);
    expect(store().files).toEqual([]);
  });

  it("records update checks", async () => {
    await store().checkUpdate();
    expect(keys().some((k) => k.startsWith("update"))).toBe(true);
  });

  it("opens the log at the right entry for a file", async () => {
    await store().addPaths(["/a/Damaged.aep"]);
    const [file] = store().files;
    store().showLogFor(file.id);
    expect(store().logOpen).toBe(true);
    expect(store().log.find((e) => e.id === store().logFocusId)?.ref).toBe(file.id);
    store().setLogOpen(false);
    expect(store().logFocusId).toBeNull();
  });
});
