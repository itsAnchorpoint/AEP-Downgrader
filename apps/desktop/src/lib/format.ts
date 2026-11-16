export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

/** "26.2" for a clean version, "26.2.1" when there is a patch. */
export function versionNumber(v: { major: number; minor: number; patch: number }): string {
  return v.patch > 0 ? `${v.major}.${v.minor}.${v.patch}` : `${v.major}.${v.minor}`;
}
