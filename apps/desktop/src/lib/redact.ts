/**
 * Path redaction for the diagnostic report. The only place that decides what a shared report may
 * reveal about the user's disk: folder names and user names never leave, a file keeps its extension
 * and gets a short tag that is the same for every mention of that file.
 *
 *   C:\Users\Jane Doe\Clients\Acme\Intro_v05.aep  ->  <file-1>.aep
 *   /Volumes/Work/Acme                            ->  <folder-1>   (when listed as a folder)
 */

export interface KnownPaths {
  /** Full paths of files the app handled. Matched literally, so spaces in names are safe. */
  files?: string[];
  /** Full paths of folders (the chosen output folder). */
  folders?: string[];
}

// Characters a path segment cannot contain on Windows; they also end a path inside a sentence.
const BAD = String.raw`"<>|?*:\r\n`;
const SEP = String.raw`[\\/]+`;
// A segment never starts or ends with a space, so "path / more text" does not run on.
const MID = String.raw`[^${BAD}\\/\s](?:[^${BAD}\\/]*[^${BAD}\\/\s])?`;
// The last segment may contain spaces if it ends in an extension ("My Comp v2.aep").
const LAST_WITH_EXT = String.raw`[^${BAD}\\/]*[^${BAD}\\/\s]\.[A-Za-z0-9]{1,5}(?![A-Za-z0-9_])`;
const LAST_PLAIN = String.raw`[^${BAD}\\/\s]+`;
const TAIL = String.raw`(?:${LAST_WITH_EXT}|${LAST_PLAIN})`;

const ROOTS = [
  String.raw`file:\/\/\/?(?:[A-Za-z]:[\\/]+)?`, // file:///home/x, file:///C:/x
  String.raw`(?<![A-Za-z0-9])[A-Za-z]:[\\/]+`, // C:\ or C:/
  String.raw`\\\\+`, // \\server\share (or the escaped form in JSON)
  String.raw`(?:~[^\s\\/"']*|%[A-Za-z_]+%|\$[A-Za-z_]+)[\\/]+`, // ~/  ~jane/  %USERPROFILE%\  $HOME/
  String.raw`(?<=^|[\s"'(\[=,;])\/+(?=[^\s\/])`, // /home/x, only where a path can begin
];

const PATH = new RegExp(`(?:${ROOTS.join("|")})(?:${MID}${SEP})*${TAIL}`, "g");
// What is left of a home directory when a path was cut short, e.g. "...\Users\jane\AppData".
const HOME_LEFTOVER = /(?<![A-Za-z0-9])(?:Users|home)[\\/]+[^\s\\/"'<>|?*]+/gi;
// Sentence punctuation that was taken into the match but does not belong to the path.
const TRAILING = /[.,;:!)\]]+$/;

const escapeRegExp = (text: string) => text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

/** Same file, same key: separators are folded, and drive paths ignore case like Windows does. */
function keyOf(path: string): string {
  const folded = path.replace(/^file:\/\/\/?/, "").replace(/[\\/]+/g, "/").replace(/\/+$/, "");
  return /^[A-Za-z]:/.test(folded) ? folded.toLowerCase() : folded;
}

function extensionOf(path: string): string {
  const last = path.split(/[\\/]+/).pop() ?? "";
  const match = /\.([A-Za-z0-9]{1,5})$/.exec(last);
  return match ? `.${match[1].toLowerCase()}` : "";
}

type Kind = "file" | "folder" | "path";

/**
 * Redacts every path in `texts`. Tags are numbered in order of first appearance across all texts,
 * so a mention in one line can be matched with a mention in another. Pure: same input, same output.
 */
export function redactAll(texts: string[], known: KnownPaths = {}): string[] {
  const tags = new Map<string, string>();
  const counters: Record<Kind, number> = { file: 0, folder: 0, path: 0 };

  function tagFor(path: string, kind: Kind): string {
    const key = keyOf(path);
    let tag = tags.get(key);
    if (!tag) {
      // A path with an extension is a file even when it was not listed.
      const ext = extensionOf(path);
      const effective: Kind = kind === "path" && ext ? "file" : kind;
      counters[effective] += 1;
      tag = `<${effective}-${counters[effective]}>${effective === "folder" ? "" : ext}`;
      tags.set(key, tag);
    }
    return tag;
  }

  const literals = [
    ...(known.files ?? []).map((path) => ({ path, kind: "file" as Kind })),
    ...(known.folders ?? []).map((path) => ({ path, kind: "folder" as Kind })),
  ]
    .filter((item) => item.path.length > 0)
    .sort((a, b) => b.path.length - a.path.length);

  return texts.map((text) => {
    let out = text;
    // Pass 1: paths the app knows, exactly as written (also with backslashes doubled, as in JSON).
    for (const { path, kind } of literals) {
      const variants = new Set([path, path.replace(/\\/g, "\\\\")]);
      for (const variant of variants) {
        // A folder also swallows what follows it ("<folder>/Intro_AE2024.aep"): that is a new path.
        const tail = kind === "folder" ? String.raw`(?:[\\/]+[^\s${BAD}]*)?` : "";
        const pattern = new RegExp(`${escapeRegExp(variant)}${tail}(?![A-Za-z0-9_])`, "g");
        out = out.replace(pattern, (match) => {
          const rest = match.length > variant.length;
          return tagFor(match, rest ? "path" : kind);
        });
      }
    }
    // Pass 2: any other path-like text.
    out = out.replace(PATH, (match) => {
      const trailing = TRAILING.exec(match)?.[0] ?? "";
      const path = trailing ? match.slice(0, -trailing.length) : match;
      return path ? tagFor(path, "path") + trailing : match;
    });
    // Pass 3: whatever still names a home directory.
    return out.replace(HOME_LEFTOVER, "<home>");
  });
}

export function redactText(text: string, known: KnownPaths = {}): string {
  return redactAll([text], known)[0];
}
