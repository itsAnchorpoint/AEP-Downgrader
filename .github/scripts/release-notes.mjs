// Usage: node .github/scripts/release-notes.mjs <tag> [--app]
//
// Checks that the repository is ready to release <tag> and prints its release notes. Fails (exit 1)
// with a reason otherwise. A beta tag (v2.0.0-beta.1) may use the section of its final version
// (2.0.0), released or not.
// - Default: the CHANGELOG.md section, for the GitHub release page.
// - --app: the short notes the app shows (apps/desktop/src/content/whatsnew.en.md, then the marker
//   line, then whatsnew.ru.md), for "notes" in the update feed. The app shows the part in its language.
import { readFileSync } from "node:fs";

const fail = (message) => {
  console.error(`release-notes: ${message}`);
  process.exit(1);
};

const tag = process.argv[2] ?? fail("usage: release-notes.mjs <tag> [--app]");
const app = process.argv.includes("--app");
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

/** `## [2.0.1] - date` → 2.0.1, `## [Unreleased] 2.0.0` → { unreleased: 2.0.0 }. */
function heading(line) {
  const match = /^##\s+\[([^\]]+)\](.*)$/.exec(line.trim());
  if (!match) return null;
  const name = match[1].trim().replace(/^v/, "");
  const dated = /\d{4}-\d{2}-\d{2}/.test(match[2]);
  if (name.toLowerCase() !== "unreleased") return { version: name, released: dated };
  const after = /\d+\.\d+\.\d+(?:-[0-9A-Za-z.]+)?/.exec(match[2])?.[0];
  return after ? { version: after, released: false } : null;
}

/** The body of the section for this tag in `file`. `needsDate`: a final release needs a dated heading. */
function section(file, needsDate) {
  const lines = readFileSync(file, "utf8").split("\n");
  const sections = [];
  lines.forEach((line, index) => {
    if (/^##\s/.test(line)) sections.push({ index, heading: heading(line) });
  });
  const accept = (h) =>
    h && ((h.version === version && (h.released || !needsDate)) || (prerelease && h.version === core));
  const at = sections.findIndex((s) => accept(s.heading));
  if (at < 0) {
    fail(
      prerelease || !needsDate
        ? `${file} has no section for ${version}${prerelease ? ` or ${core}` : ""}`
        : `${file} has no released section "## [${version}] - YYYY-MM-DD"`,
    );
  }
  const end = sections[at + 1]?.index ?? lines.length;
  const body = lines.slice(sections[at].index + 1, end).join("\n").trim();
  if (!body) fail(`the ${file} section for ${sections[at].heading.version} is empty`);
  return body;
}

// Both checks always run, so neither file can be forgotten before a release.
const changelog = section("CHANGELOG.md", true);
const en = section("apps/desktop/src/content/whatsnew.en.md", false);
const ru = section("apps/desktop/src/content/whatsnew.ru.md", false);
process.stdout.write(app ? `${en}\n\n<!-- lang:ru -->\n\n${ru}\n` : `${changelog}\n`);
