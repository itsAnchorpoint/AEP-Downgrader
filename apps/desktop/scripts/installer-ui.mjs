// Builds the window of the Windows installer: the NSIS plug-in in src-tauri/windows/ui, copied to
// src-tauri/windows/plugin/aepd_ui.dll, where the installer template (windows/hooks.nsh) finds it.
// Tauri runs this before bundling (build.beforeBundleCommand); it does nothing unless the bundle
// is for Windows (pass --force to build it anyway, e.g. for a local test on Linux with MinGW).
// NSIS is 32-bit, so the plug-in is too; the C runtime is linked in (ui/.cargo/config.toml).
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";

const platform = process.env.TAURI_ENV_PLATFORM;
const force = process.argv.includes("--force");
if (!force && (platform ? platform !== "windows" : process.platform !== "win32")) process.exit(0);

execFileSync(process.execPath, [fileURLToPath(new URL("check-installer-template.mjs", import.meta.url))], { stdio: "inherit" });

const target = process.platform === "win32" ? "i686-pc-windows-msvc" : "i686-pc-windows-gnu";
const ui = fileURLToPath(new URL("../src-tauri/windows/ui/", import.meta.url));
const plugin = fileURLToPath(new URL("../src-tauri/windows/plugin/", import.meta.url));
const run = (cmd, args) => execFileSync(cmd, args, { cwd: ui, stdio: "inherit" });

run("rustup", ["target", "add", target]);
run("cargo", ["build", "--release", "--locked", "--target", target]);
mkdirSync(plugin, { recursive: true });
copyFileSync(`${ui}target/${target}/release/aepd_ui.dll`, `${plugin}aepd_ui.dll`);
console.log(`installer window plug-in: ${plugin}aepd_ui.dll (${target})`);
