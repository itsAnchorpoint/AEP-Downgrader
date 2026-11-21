/** Semantic version order: 2.0.0-beta.1 < 2.0.0-beta.2 < 2.0.0 < 2.0.1. Unparsable parts count as 0. */
export function compareVersions(a: string, b: string): number {
  const split = (v: string) => {
    const [core, pre] = v.trim().replace(/^v/, "").split(/-(.*)/s, 2);
    return { core: core.split(".").map((n) => Number.parseInt(n, 10) || 0), pre: pre ? pre.split(".") : [] };
  };
  const x = split(a);
  const y = split(b);
  for (let i = 0; i < 3; i++) {
    const d = (x.core[i] ?? 0) - (y.core[i] ?? 0);
    if (d !== 0) return Math.sign(d);
  }
  if (x.pre.length === 0 || y.pre.length === 0) return Math.sign(y.pre.length - x.pre.length);
  for (let i = 0; i < Math.max(x.pre.length, y.pre.length); i++) {
    const p = x.pre[i];
    const q = y.pre[i];
    if (p === undefined) return -1;
    if (q === undefined) return 1;
    const np = /^\d+$/.test(p);
    const nq = /^\d+$/.test(q);
    if (np && nq && Number(p) !== Number(q)) return Math.sign(Number(p) - Number(q));
    if (np !== nq) return np ? -1 : 1;
    if (p !== q) return p < q ? -1 : 1;
  }
  return 0;
}

/**
 * Version of a `## [x.y.z] - date` heading. `## [Unreleased] x.y.z` gives x.y.z; a heading
 * without a version gives null.
 */
export function headingVersion(line: string): string | null {
  const match = /^##\s+\[([^\]]+)\](.*)$/.exec(line.trim());
  if (!match) return null;
  const inBrackets = match[1].trim();
  if (inBrackets.toLowerCase() !== "unreleased") return inBrackets.replace(/^v/, "");
  return /\d+\.\d+\.\d+(?:-[0-9A-Za-z.]+)?/.exec(match[2])?.[0] ?? null;
}

/**
 * The release notes a user has not seen: every version section newer than `since` (all of them
 * when `since` is null). Anything before the first section, such as an introduction, is left out,
 * and so are sections whose heading has no version when `since` is given.
 */
export function changelogSince(markdown: string, since: string | null): string {
  if (since === null) {
    const first = markdown.search(/^##\s/m);
    return first < 0 ? markdown : markdown.slice(first);
  }
  const lines = markdown.split("\n");
  const out: string[] = [];
  let keep = true; // the title before the first section
  let sawSection = false;
  for (const line of lines) {
    if (/^##\s/.test(line)) {
      sawSection = true;
      const version = headingVersion(line);
      keep = version !== null && compareVersions(version, since) > 0;
    } else if (!sawSection) {
      keep = false;
    }
    if (keep) out.push(line);
  }
  return out.join("\n").trim() + "\n";
}

/** True if the notes have at least one section newer than `since`. */
export function hasChangesSince(markdown: string, since: string): boolean {
  return markdown.split("\n").some((line) => {
    const version = /^##\s/.test(line) ? headingVersion(line) : null;
    return version !== null && compareVersions(version, since) > 0;
  });
}

/** Marker between the English and the Russian notes in the update feed (see release-notes.mjs). */
export const RU_NOTES_MARKER = "<!-- lang:ru -->";

/** The part of the feed's notes in the app's language; English when there is no Russian part. */
export function notesFor(notes: string, language: string): string {
  const [en, ru] = notes.split(RU_NOTES_MARKER);
  return (language === "ru" && ru?.trim() ? ru : en).trim();
}
