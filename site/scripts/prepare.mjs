// Runs before every dev, build and check. It takes from the rest of the repository what the site shows,
// so the site cannot drift from the app:
// - the brand files (fonts, grain, the Fold ribbon, the icon at 256 px) into public/brand/;
// - the core built for the browser into public/wasm/aep.wasm (needs the wasm32-unknown-unknown target);
// - src/generated/data.json: the app version and its download links, the compatibility table as the
//   core prints it (`aep compat`), and the number of downloads of the app on GitHub (also written to
//   public/badge/downloads.json for the README badge).
// The download count needs the network; without it the last known count is kept, or the line is left out.
// With SITE_STRICT=1 (the deploy workflow) a core that fails to build stops the build instead.
import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const site = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(site, "..");
const REPO = "itsAnchorpoint/AEP-Downgrader";

// Brand files, copied rather than duplicated in git.
const brand = join(site, "public/brand");
mkdirSync(join(brand, "fonts"), { recursive: true });
const assets = join(repo, "apps/desktop/src/assets");
for (const font of [
  "onest-latin-wght-normal.woff2",
  "onest-cyrillic-wght-normal.woff2",
  "jetbrains-mono-latin-wght-normal.woff2",
  "jetbrains-mono-cyrillic-wght-normal.woff2",
]) {
  copyFileSync(join(assets, "fonts", font), join(brand, "fonts", font));
}
copyFileSync(join(assets, "grain.png"), join(brand, "grain.png"));
copyFileSync(join(assets, "fold-ribbon.svg"), join(brand, "fold-ribbon.svg"));
copyFileSync(join(repo, "apps/desktop/src-tauri/icons/128x128@2x.png"), join(brand, "icon.png"));

// The core for the "check a file" section, built for the browser (crates/aep-wasm).
try {
  execFileSync("cargo", ["build", "--quiet", "--release", "-p", "aep-wasm", "--target", "wasm32-unknown-unknown"], {
    cwd: repo,
    stdio: "inherit",
  });
  mkdirSync(join(site, "public/wasm"), { recursive: true });
  copyFileSync(join(repo, "target/wasm32-unknown-unknown/release/aep_wasm.wasm"), join(site, "public/wasm/aep.wasm"));
} catch (error) {
  if (process.env.SITE_STRICT === "1") throw error;
  console.warn(`prepare: the WebAssembly core was not built (${error.message}); the file check will not work`);
}

// The version the download buttons point at, and the release file names it gives.
const version = JSON.parse(readFileSync(join(repo, "apps/desktop/package.json"), "utf8")).version;
const base = `https://github.com/${REPO}/releases/download/v${version}`;
const downloads = {
  windowsSetup: `${base}/AEP.Downgrader_${version}_x64-setup.exe`,
  windowsPortable: `${base}/AEP-Downgrader_x64-portable.exe`,
  macos: `${base}/AEP.Downgrader_${version}_universal.dmg`,
  appimage: `${base}/AEP.Downgrader_${version}_amd64.AppImage`,
  deb: `${base}/AEP.Downgrader_${version}_amd64.deb`,
  checksums: `${base}/SHA256SUMS.txt`,
  release: `https://github.com/${REPO}/releases/tag/v${version}`,
};

// The compatibility table, from the core: | After Effects 2026 (26.x) | Experimental | basis |
const table = execFileSync("cargo", ["run", "--quiet", "-p", "aep-cli", "--", "compat"], { cwd: repo, encoding: "utf8" });
const targets = table
  .trim()
  .split("\n")
  .slice(2)
  .map((row) => {
    const [target, status, basis] = row.split("|").slice(1, 4).map((cell) => cell.trim());
    const [, year, major] = /After Effects (\d{4}) \((\d+)\.x\)/.exec(target) ?? [];
    return { year: Number(year), major: Number(major), status: status.toLowerCase(), basis };
  });

// Downloads of the app itself, rounded down for the page ("11,000+"). Files the app fetches by itself
// are left out, or the number would grow with every daily update check: latest.json, signatures,
// checksums, and the macOS update bundle (.app.tar.gz). Installers updating themselves still count.
const isAppFile = (name) => !/\.(sig|json|txt)$|\.app\.tar\.gz$/.test(name);
const out = join(site, "src/generated/data.json");
const previous = existsSync(out) ? JSON.parse(readFileSync(out, "utf8")) : {};
let downloadCount = previous.downloadCount ?? null;
try {
  const headers = { Accept: "application/vnd.github+json", "User-Agent": "aepd-site-build" };
  if (process.env.GITHUB_TOKEN) headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  let total = 0;
  for (let page = 1; page < 10; page++) {
    const res = await fetch(`https://api.github.com/repos/${REPO}/releases?per_page=100&page=${page}`, {
      headers,
      signal: AbortSignal.timeout(10_000),
    });
    if (!res.ok) throw new Error(`GitHub answered ${res.status}`);
    const releases = await res.json();
    for (const release of releases)
      for (const asset of release.assets) if (isAppFile(asset.name)) total += asset.download_count;
    if (releases.length < 100) break;
  }
  downloadCount = total;
} catch (error) {
  console.warn(`prepare: download count not updated (${error.message})`);
}

// The same number for the README badge (shields.io reads it from the site: /badge/downloads.json).
if (downloadCount !== null) {
  mkdirSync(join(site, "public/badge"), { recursive: true });
  const message = downloadCount >= 1000 ? `${Math.floor(downloadCount / 1000)}k` : String(downloadCount);
  const badge = { schemaVersion: 1, label: "downloads", message, color: "d6ff3d", labelColor: "1d1d21" };
  writeFileSync(join(site, "public/badge/downloads.json"), JSON.stringify(badge) + "\n");
}

// The app's own words for the live window on the home page: the same labels and report notes.
const appStrings = {};
for (const lang of ["en", "ru"]) {
  const all = JSON.parse(readFileSync(join(repo, `apps/desktop/src/i18n/${lang}.json`), "utf8"));
  appStrings[lang] = {
    drop: all.drop,
    queue: all.queue,
    file: all.file,
    platform: all.platform,
    problem: all.problem,
    target: all.target,
    report: all.report,
    note: all.note,
    state: all.state,
    converting: all.converting,
    done: all.done,
  };
}
mkdirSync(dirname(out), { recursive: true });
writeFileSync(join(site, "src/generated/app-strings.json"), JSON.stringify(appStrings, null, 2) + "\n");
writeFileSync(out, JSON.stringify({ version, downloads, targets, downloadCount }, null, 2) + "\n");
console.log(`prepare: version ${version}, ${targets.length} targets, ${downloadCount ?? "no"} downloads`);
