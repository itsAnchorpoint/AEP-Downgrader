// The Windows installer template (src-tauri/windows/installer.nsi) is a copy of Tauri's own, so
// Tauri updates do not reach it by themselves. This check fails when the project uses another
// tauri-cli (which renders the template) or tauri-plugin-updater (which runs the installer to
// update the app) than the versions the template was last checked against.
// Runs in CI and before every Windows bundle (scripts/installer-ui.mjs).
import { readFileSync } from "node:fs";

const read = (path) => readFileSync(new URL(path, import.meta.url), "utf8");
const template = read("../src-tauri/windows/installer.nsi");
const checked = template.match(/Checked against: tauri-cli ([\w.-]+), tauri-plugin-updater ([\w.-]+)/);
if (!checked) throw new Error("installer.nsi: the 'Checked against' line is missing");

const cli = JSON.parse(read("../node_modules/@tauri-apps/cli/package.json")).version;
const lock = read("../src-tauri/Cargo.lock");
const updater = lock.match(/name = "tauri-plugin-updater"\r?\nversion = "([^"]+)"/)?.[1];

const problems = [];
if (cli !== checked[1]) problems.push(`tauri-cli is ${cli}, the template was checked against ${checked[1]}`);
if (updater !== checked[2]) problems.push(`tauri-plugin-updater is ${updater}, the template was checked against ${checked[2]}`);
if (problems.length) {
  console.error(`The Windows installer template needs a look:\n- ${problems.join("\n- ")}
Compare apps/desktop/src-tauri/windows/installer.nsi with Tauri's template:
  https://github.com/tauri-apps/tauri/compare/tauri-cli-v${checked[1]}...tauri-cli-v${cli}
  (crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi and utils.nsh)
and the updater's installer arguments:
  https://github.com/tauri-apps/plugins-workspace/compare/updater-v${checked[2]}...updater-v${updater}
  (plugins/updater/src/updater.rs, config.rs).
Port what changed, update the "Checked against" line, and test an update from the app on Windows.`);
  process.exit(1);
}
console.log(`installer template checked against tauri-cli ${cli}, tauri-plugin-updater ${updater}`);
