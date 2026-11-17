// Usage: node .github/scripts/finish-feed.mjs <latest.json> <notes.md> <portable.exe.sig> <portable url> [assets.json]
//
// Completes the update feed that tauri-action wrote: adds the Windows portable exe, sets the
// release notes, and checks that every platform the app ships is present. Rewrites <latest.json>.
// tauri-action points the files of a draft release at API URLs (api.github.com/.../assets/<id>);
// with assets.json (`gh release view --json assets`) they become the normal download links that
// the release will have once published.
import { readFileSync, writeFileSync } from "node:fs";

const [feedPath, notesPath, sigPath, portableUrl, assetsPath] = process.argv.slice(2);
if (!portableUrl) {
  console.error("usage: finish-feed.mjs <latest.json> <notes.md> <portable.exe.sig> <portable url>");
  process.exit(1);
}

const feed = JSON.parse(readFileSync(feedPath, "utf8"));
feed.notes = readFileSync(notesPath, "utf8").trim();
feed.platforms ??= {};
feed.platforms["windows-x86_64-portable"] = {
  signature: readFileSync(sigPath, "utf8").trim(),
  url: portableUrl,
};

if (assetsPath) {
  const base = portableUrl.slice(0, portableUrl.lastIndexOf("/") + 1);
  const byApiUrl = new Map(JSON.parse(readFileSync(assetsPath, "utf8")).assets.map((a) => [a.apiUrl, a.name]));
  for (const entry of Object.values(feed.platforms)) {
    const name = byApiUrl.get(entry.url);
    if (name) entry.url = base + encodeURIComponent(name);
  }
}
const stale = Object.entries(feed.platforms).filter(([, e]) => e.url.startsWith("https://api.github.com/"));
if (assetsPath && stale.length) {
  console.error(`finish-feed: no download link for ${stale.map(([k]) => k).join(", ")}`);
  process.exit(1);
}

const required = ["windows-x86_64", "windows-x86_64-portable", "darwin-x86_64", "darwin-aarch64", "linux-x86_64"];
const missing = required.filter((key) => !feed.platforms[key]?.url || !feed.platforms[key]?.signature);
if (missing.length) {
  console.error(`finish-feed: the feed has no ${missing.join(", ")}`);
  process.exit(1);
}

writeFileSync(feedPath, `${JSON.stringify(feed, null, 2)}\n`);
console.log(`Feed ${feed.version}: ${Object.keys(feed.platforms).sort().join(", ")}`);
