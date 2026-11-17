// Usage: node .github/scripts/release-notes.mjs <tag>
//
// Checks that the repository is ready to release <tag> and prints the release notes: the
// CHANGELOG.md section of that version. Fails (exit 1) with a reason otherwise.
// A beta tag (v2.0.0-beta.1) may use the section of its final version (2.0.0), released or not.
import { readFileSync } from "node:fs";

const fail = (message) => {
  console.error(`release-notes: ${message}`);
  process.exit(1);
};

const tag = process.argv[2] ?? fail("usage: release-notes.mjs <tag>");
const version = tag.replace(/^v/, "");
if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.]+)?$/.test(version)) fail(`"${tag}" is not a version tag like v2.0.0`);
const core = version.split("-")[0];
const prerelease = version !== core;

const pkg = JSON.parse(readFileSync("apps/desktop/package.json", "utf8")).version;
const cargo = /^version\s*=\s*"([^"]+)"/m.exec(readFileSync("apps/desktop/src-tauri/Cargo.toml", "utf8"))?.[1];
if (pkg !== version) fail(`apps/desktop/package.json has version ${pkg}, the tag is ${version}`);
if (cargo !== version) fail(`apps/desktop/src-tauri/Cargo.toml has version ${cargo}, the tag is ${version}`);

const conf = JSON.parse(readFileSync("apps/desktop/src-tauri/tauri.conf.json", "utf8"));
const pubkey = conf.plugins?.updater?.pubkey ?? "";
if (!pubkey || pubkey.startsWith("REPLACE")) fail("tauri.conf.json has no updater public key");

const changelog = readFileSync("CHANGELOG.md", "utf8");
if (readFileSync("apps/desktop/src/content/CHANGELOG.md", "utf8") !== changelog) {
  fail("apps/desktop/src/content/CHANGELOG.md differs from CHANGELOG.md; copy it over");
}

/** `## [2.0.1] - date` → 2.0.1, `## [Unreleased] 2.0.0` → { unreleased: 2.0.0 }. */
function heading(line) {
  const match = /^##\s+\[([^\]]+)\](.*)$/.exec(line.trim());
  if (!match) return null;
  const name = match[1].trim().replace(/^v/, "");
  if (name.toLowerCase() !== "unreleased") return { version: name, released: true };
  const after = /\d+\.\d+\.\d+(?:-[0-9A-Za-z.]+)?/.exec(match[2])?.[0];
  return after ? { version: after, released: false } : null;
}

const lines = changelog.split("\n");
const sections = [];
lines.forEach((line, index) => {
  if (/^##\s/.test(line)) sections.push({ index, heading: heading(line) });
});
const accept = (h) =>
  h && ((h.version === version && h.released) || (prerelease && h.version === core));
const at = sections.findIndex((s) => accept(s.heading));
if (at < 0) {
  fail(
    prerelease
      ? `CHANGELOG.md has no section for ${version} or ${core}`
      : `CHANGELOG.md has no released section "## [${version}] - YYYY-MM-DD"`,
  );
}
const end = sections[at + 1]?.index ?? lines.length;
const body = lines.slice(sections[at].index + 1, end).join("\n").trim();
if (!body) fail(`the CHANGELOG.md section for ${sections[at].heading.version} is empty`);
process.stdout.write(`${body}\n`);
