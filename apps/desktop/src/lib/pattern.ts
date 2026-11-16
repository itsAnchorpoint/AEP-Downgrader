export const DEFAULT_PATTERN = "{name}_AE{year}";

/**
 * Builds the output file name from the user's pattern.
 * `{name}` is the original name without extension, `{year}` is the target version name.
 */
export function outputName(pattern: string, fileName: string, year: number | string): string {
  const dot = fileName.lastIndexOf(".");
  const stem = dot > 0 ? fileName.slice(0, dot) : fileName;
  const ext = dot > 0 ? fileName.slice(dot) : "";
  const fill = (p: string) => p.replaceAll("{name}", stem).replaceAll("{year}", String(year));
  const clean = (s: string) =>
    // Characters Windows forbids in file names, and trailing dots/spaces.
    s.replace(/[<>:"/\\|?*\u0000-\u001f]/g, "_").replace(/[. ]+$/, "");
  const base = clean(fill(pattern.trim() || DEFAULT_PATTERN)) || clean(fill(DEFAULT_PATTERN));
  return base + ext;
}
