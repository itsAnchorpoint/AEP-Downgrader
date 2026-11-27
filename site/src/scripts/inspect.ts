// The core in WebAssembly (crates/aep-wasm): identify a file from its first 64 KiB, in this page.
// The file is read by the browser and never sent anywhere.

export interface Inspection {
  kind: "project" | "preset" | "xml" | "sidecar" | "unknown";
  problem: string | null;
  version: { major: number; minor: number; patch: number; build: number; platform: string; release: boolean; name: string | null } | null;
  format: { major: number; minor: number } | null;
  release: string | null;
  targets: { major: number; name: string | null; status: "verified" | "experimental" }[];
}

interface Exports {
  memory: WebAssembly.Memory;
  alloc(len: number): number;
  inspect(ptr: number, len: number): void;
  result_ptr(): number;
  result_len(): number;
}

const PREFIX = 64 * 1024;
let core: Promise<Exports> | null = null;

function load(): Promise<Exports> {
  core ??= (async () => {
    const response = await fetch("/wasm/aep.wasm");
    if (!response.ok) throw new Error(`core: ${response.status}`);
    const { instance } = await WebAssembly.instantiate(await response.arrayBuffer(), {});
    return instance.exports as unknown as Exports;
  })();
  return core;
}

export async function inspectFile(file: File): Promise<Inspection> {
  const [wasm, bytes] = await Promise.all([load(), file.slice(0, PREFIX).arrayBuffer()]);
  const ptr = wasm.alloc(bytes.byteLength);
  new Uint8Array(wasm.memory.buffer, ptr, bytes.byteLength).set(new Uint8Array(bytes));
  wasm.inspect(ptr, bytes.byteLength);
  const out = new Uint8Array(wasm.memory.buffer, wasm.result_ptr(), wasm.result_len());
  return JSON.parse(new TextDecoder().decode(out));
}

export interface Report {
  target: number;
  blocked: boolean;
  notes: { severity: "info" | "warn" | "block" | "ok"; code: string; params: Record<string, unknown> }[];
}

interface ReportExports extends Exports {
  preflight(ptr: number, len: number, target: number): void;
}

// The whole file, read once per file: the report converts it in memory and throws the result away.
const whole = new WeakMap<File, Promise<ArrayBuffer>>();

export async function reportFile(file: File, target: number): Promise<Report> {
  if (!whole.has(file)) whole.set(file, file.arrayBuffer());
  const [wasm, bytes] = await Promise.all([load() as Promise<ReportExports>, whole.get(file)!]);
  const ptr = wasm.alloc(bytes.byteLength);
  new Uint8Array(wasm.memory.buffer, ptr, bytes.byteLength).set(new Uint8Array(bytes));
  wasm.preflight(ptr, bytes.byteLength, target);
  const out = new Uint8Array(wasm.memory.buffer, wasm.result_ptr(), wasm.result_len());
  return JSON.parse(new TextDecoder().decode(out));
}
