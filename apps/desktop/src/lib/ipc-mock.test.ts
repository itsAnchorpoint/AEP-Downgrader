import { describe, expect, it } from "vitest";
import { createMockIpc } from "./ipc-mock";
import type { ProgressEvent } from "./ipc-types";

describe("mock ipc", () => {
  it("inspects files and flags problems from the name", async () => {
    const ipc = createMockIpc(0);
    const [project, sidecar, other] = await ipc.inspectPaths(["/a/Intro.aep", "/a/._Intro.aep", "/a/notes.txt"]);
    expect(project.kind).toBe("project");
    expect(project.version?.name).toMatch(/^20\d\d$/);
    expect(project.problem).toBeUndefined();
    expect(sidecar).toMatchObject({ kind: "sidecar", problem: "sidecar" });
    expect(other.problem).toBe("not-ae-file");
  });

  it("gives the same answer for the same path", async () => {
    const ipc = createMockIpc(0);
    expect(await ipc.inspectPaths(["/a/Intro.aep"])).toEqual(await ipc.inspectPaths(["/a/Intro.aep"]));
  });

  it("lists targets newest first with experimental ones last", async () => {
    const targets = await createMockIpc(0).listTargets();
    expect(targets.map((t) => t.major)).toEqual([26, 25, 24, 23, 22]);
    expect(targets.at(-1)?.status).toBe("experimental");
  });

  it("blocks a target that is not older than the file", async () => {
    const ipc = createMockIpc(0);
    const [file] = await ipc.inspectPaths(["/a/Intro.aep"]);
    const report = await ipc.preflight(file.path, file.version!.major);
    expect(report.blocked).toBe(true);
    expect(report.notes[0].code).toBe("target-not-older");
  });

  it("emits queued, running and done events with an output path", async () => {
    const ipc = createMockIpc(0);
    const events: ProgressEvent[] = [];
    await ipc.onProgress((e) => events.push(e));
    await ipc.convert([{ id: "1", path: "/a/Intro.aep", target: 22 }], {
      location: "next-to-source",
      pattern: "{name}_AE{year}",
      onExists: "rename",
    });
    await new Promise((resolve) => setTimeout(resolve, 10));
    expect(events.map((e) => e.state)).toEqual(["queued", "running", "done"]);
    expect(events[2].outputPath).toBe("/a/Intro_AE2022.aep");
  });

  it("fails files whose name asks for it", async () => {
    const ipc = createMockIpc(0);
    const events: ProgressEvent[] = [];
    await ipc.onProgress((e) => events.push(e));
    await ipc.convert([{ id: "1", path: "/a/fail.aep", target: 22 }], {
      location: { folder: "/out" },
      pattern: "",
      onExists: "skip",
    });
    await new Promise((resolve) => setTimeout(resolve, 10));
    expect(events.at(-1)).toMatchObject({ state: "failed", errorCode: "write-failed" });
  });
});
