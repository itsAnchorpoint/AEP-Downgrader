export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

/** "26.2" for a clean version, "26.2.1" when there is a patch. */
/** After Effects major for a file format major: 94 → 23 … 98 → 27. Null outside that range (93 is shared by 16–22). */
export function aeMajorForFormat(major: number | undefined): number | null {
  return major !== undefined && major >= 94 && major <= 98 ? major - 71 : null;
}

export function versionNumber(v: { major: number; minor: number; patch: number }): string {
  return v.patch > 0 ? `${v.major}.${v.minor}.${v.patch}` : `${v.major}.${v.minor}`;
}
