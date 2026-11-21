// Development: screenshots of every app state with the mock backend (no Tauri needed).
//   node scripts/shots.mjs <out dir> [states] [themes] [langs] [WxH]
// Lists are comma-separated. Needs Chromium and the Vite dev server on :1420
// (scripts/shots.sh starts it). Drives Chromium over the DevTools protocol and waits until
// the demo state reports itself ready (<html data-demo-ready>), so no shot is taken early.
import { spawn } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const [out, statesArg, themesArg, langsArg, sizeArg] = process.argv.slice(2);
if (!out) {
  console.error("usage: node scripts/shots.mjs <out dir> [states] [themes] [langs] [WxH]");
  process.exit(2);
}
const list = (arg, fallback) => (arg ? arg.split(",") : fallback);
const states = list(statesArg, ["empty", "queue", "report", "converting", "done", "settings", "update", "installing", "installed", "whatsnew", "history", "log"]);
const themes = list(themesArg, ["dark", "light", "ae"]);
const langs = list(langsArg, ["en", "ru"]);
const [width, height] = (sizeArg ?? "920x680").split("x").map(Number);
// Optional extra query (SHOT_QUERY="&v=a") and a file name tag (SHOT_TAG=a) for design variants.
const extra = process.env.SHOT_QUERY ?? "";
const tag = process.env.SHOT_TAG ? `-${process.env.SHOT_TAG}` : "";

const profile = mkdtempSync(join(tmpdir(), "aepd-shots-"));
const port = 9300 + Math.floor(Math.random() * 500);
const browser = spawn("chromium", [
  "--headless=new", "--disable-gpu", "--hide-scrollbars", "--force-prefers-reduced-motion",
  `--user-data-dir=${profile}`, `--remote-debugging-port=${port}`, "--lang=en-US", "about:blank",
], { stdio: "ignore" });

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let ws;
for (let i = 0; i < 50 && !ws; i++) {
  await sleep(200);
  try {
    const pages = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
    const page = pages.find((p) => p.type === "page");
    if (page) ws = new WebSocket(page.webSocketDebuggerUrl);
  } catch {}
}
await new Promise((r) => ws.addEventListener("open", r, { once: true }));

let nextId = 1;
const pending = new Map();
ws.addEventListener("message", (e) => {
  const msg = JSON.parse(e.data);
  if (msg.id && pending.has(msg.id)) {
    pending.get(msg.id)(msg);
    pending.delete(msg.id);
  }
});
const send = (method, params = {}) =>
  new Promise((resolve) => {
    const id = nextId++;
    pending.set(id, resolve);
    ws.send(JSON.stringify({ id, method, params }));
  });
const evaluate = async (expression) => (await send("Runtime.evaluate", { expression, returnByValue: true })).result?.result?.value;

await send("Page.enable");
await send("Emulation.setDeviceMetricsOverride", { width, height, deviceScaleFactor: 2, mobile: false });
let count = 0;
for (const state of states) for (const theme of themes) for (const lang of langs) {
  // Each shot starts from empty storage, so settings never leak from one shot to the next.
  await send("Storage.clearDataForOrigin", { origin: "http://localhost:1420", storageTypes: "all" });
  await send("Page.navigate", { url: `http://localhost:1420/?demo=${state}&theme=${theme}&lang=${lang}${extra}` });
  let ready = false;
  for (let i = 0; i < 100 && !ready; i++) {
    await sleep(100);
    ready = (await evaluate("document.documentElement.dataset.demoReady")) === state;
  }
  if (!ready) console.error(`not ready: ${state} ${theme} ${lang}`);
  await sleep(400); // fonts and the last paint
  const shot = await send("Page.captureScreenshot", { format: "png" });
  writeFileSync(join(out, `${state}-${theme}-${lang}-${width}x${height}${tag}.png`), Buffer.from(shot.result.data, "base64"));
  count++;
}
ws.close();
const exited = new Promise((r) => browser.once("exit", r));
browser.kill();
await exited;
rmSync(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
console.log(`${count} shots in ${out}`);
