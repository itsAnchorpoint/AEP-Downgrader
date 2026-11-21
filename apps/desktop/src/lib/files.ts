import type { FileInfo, Problem } from "./ipc-types";

/** Problems that only mean "this is not a project": shown quietly, not as errors. */
const IGNORED: Problem[] = ["sidecar", "not-ae-file"];

export function isIgnored(file: FileInfo) {
  return file.problem !== undefined && IGNORED.includes(file.problem);
}
