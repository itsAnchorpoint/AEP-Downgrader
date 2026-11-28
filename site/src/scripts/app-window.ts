// The live app window in the hero (components/AppWindow.astro).
// 1. It scales the 920 x 640 window to its column.
// 2. It plays the whole job once: a file is dropped, 2024 is picked, the report opens, Convert, done.
//    The three steps under the window light up as it goes. Then it hands over: "your turn".
// 3. A file the visitor drops (or chooses) is read in this page by the core in WebAssembly: its
//    version, the versions it converts to, and the real report for the chosen one. Nothing is uploaded;
//    the copy itself is made by the app.
import strings from "../generated/app-strings.json";
import { inspectFile, reportFile, type Inspection, type Report } from "./inspect";

interface SiteText {
  neverLeft: string;
  downloadToConvert: string;
  bigFile: string;
  bigFileSummary: string;
  reportFailed: string;
}

// The report converts the whole file in this tab's memory. Above this size the page checks only the
// version, so a big but healthy project never looks broken here.
const REPORT_LIMIT = 300 * 1024 * 1024;

const stage = document.querySelector<HTMLElement>("[data-appwin]");
if (stage) init(stage);

function init(stage: HTMLElement) {
  const lang = (stage.dataset.lang ?? "en") as "en" | "ru";
  const app = strings[lang] as Record<string, unknown>;
  const site: SiteText = JSON.parse(stage.closest<HTMLElement>("[data-site]")?.dataset.site ?? "{}");
  const plural = new Intl.PluralRules(lang);
  const pick = (key: string): string | null => {
    const value = key.split(".").reduce<unknown>((o, k) => (o as Record<string, unknown> | undefined)?.[k], app);
    return typeof value === "string" ? value : null;
  };
  const say = (key: string, vars: Record<string, string | number> = {}) => {
    const count = typeof vars.count === "number" ? vars.count : null;
    const text = (count !== null ? (pick(`${key}_${plural.select(count)}`) ?? pick(`${key}_other`)) : null) ?? pick(key) ?? key;
    return text.replace(/\{\{(\w+)\}\}/g, (_, k) => String(vars[k] ?? ""));
  };

  const win = stage.querySelector<HTMLElement>(".aw")!;
  const frame = stage.querySelector<HTMLElement>(".aw-frame")!;
  const $ = <T extends Element = HTMLElement>(sel: string) => win.querySelector<T>(sel)!;
  const steps = [...stage.querySelectorAll<HTMLElement>("[data-aw-step]")];
  const replay = stage.querySelector<HTMLButtonElement>("[data-aw-replay]")!;
  const input = stage.querySelector<HTMLInputElement>("[data-aw-input]")!;
  const pointer = $(".aw-pointer");
  const drag = $(".aw-drag");
  const tiles = [...win.querySelectorAll<HTMLElement>("[data-aw-tile]")];
  const tileRow = $(".aw-tiles");
  const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const modes: Record<string, [string, string]> = JSON.parse(stage.dataset.modes ?? "{}");
  const modeLabel = stage.querySelector("[data-aw-mode]");
  const modeHint = stage.querySelector("[data-aw-mode-hint]");
  const progress = stage.querySelector<HTMLElement>("[data-aw-progress]");
  const another = $<HTMLButtonElement>("[data-aw-another]");
  const live = stage.querySelector<HTMLElement>("[data-aw-live]");
  // Keyboard and screen readers: the tiles are radio buttons, reachable only while the visitor's own file is
  // shown (one tab stop on the chosen tile, arrows between the others).
  const syncTiles = () => {
    const own = stage.dataset.mode === "own";
    const usable = tiles.filter((t) => !t.classList.contains("off"));
    const stop = usable.find((t) => t.classList.contains("on")) ?? usable[0];
    for (const t of tiles) {
      t.setAttribute("aria-checked", String(t.classList.contains("on")));
      t.setAttribute("aria-disabled", String(t.classList.contains("off")));
      t.tabIndex = own && t === stop ? 0 : -1;
    }
  };
  // While the demo plays or waits, the whole window is one "choose a file" button. With the visitor's own
  // file it becomes a group of real controls: the tiles and "Add more" for another file.
  const setMode = (mode: "demo" | "turn" | "own") => {
    stage.dataset.mode = mode;
    if (modeLabel) modeLabel.textContent = modes[mode]?.[0] ?? "";
    if (modeHint) modeHint.textContent = modes[mode]?.[1] ?? "";
    const own = mode === "own";
    frame.setAttribute("role", own ? "group" : "button");
    frame.setAttribute("aria-label", own ? (modes.own?.[0] ?? "") : (modes.turn?.[1] ?? ""));
    frame.tabIndex = own ? -1 : 0;
    another.tabIndex = own ? 0 : -1;
    if (live && !own) live.textContent = "";
    syncTiles();
  };

  // 1. Scale to the column.
  const fit = () => win.style.setProperty("--k", String(frame.clientWidth / 920));
  new ResizeObserver(fit).observe(frame);
  fit();

  // The demo's starting point: what the server rendered (the report of the sample project).
  const sample = {
    row: $("[data-aw-row]").innerHTML,
    report: $("[data-aw-report]").innerHTML,
    save: $("[data-aw-save]").innerHTML,
    convert: $("[data-aw-convert]").outerHTML,
  };
  const setState = (state: string) => {
    win.dataset.state = state;
    for (const screen of win.querySelectorAll<HTMLElement>(".aw-screen")) {
      screen.setAttribute("aria-hidden", String(!screen.classList.contains(`aw-${state}`)));
    }
  };
  const lightStep = (n: number) => steps.forEach((s, i) => s.classList.toggle("on", i + 1 === n));
  const choose = (major: number | null) => {
    tiles.forEach((t) => t.classList.toggle("on", Number(t.dataset.awTile) === major));
    syncTiles();
  };
  const restoreSample = () => {
    $("[data-aw-row]").innerHTML = sample.row;
    $("[data-aw-report]").innerHTML = sample.report;
    $("[data-aw-save]").innerHTML = sample.save;
    $("[data-aw-convert]").outerHTML = sample.convert;
    tiles.forEach((t) => t.classList.remove("off"));
    tileRow.classList.remove("pickable");
    syncTiles();
  };

  // 2. The demo.
  let run = 0;
  const sleep = (ms: number, id: number) =>
    new Promise<void>((resolve, reject) => setTimeout(() => (id === run ? resolve() : reject(new Error("stopped"))), ms));
  const move = (el: HTMLElement, from: [number, number], to: [number, number], ms: number) =>
    el.animate([{ transform: `translate(${from[0]}px, ${from[1]}px)` }, { transform: `translate(${to[0]}px, ${to[1]}px)` }], {
      duration: ms,
      easing: "cubic-bezier(0.2, 0.9, 0.2, 1)",
      fill: "forwards",
    }).finished;
  const show = (el: HTMLElement, on: boolean) => el.animate([{ opacity: on ? 1 : 0 }], { duration: 200, fill: "forwards" });

  const DEMO_MS = 11000;
  async function demo() {
    const id = ++run;
    replay.hidden = true;
    current = null;
    setMode("demo");
    restoreSample();
    progress?.getAnimations().forEach((a) => a.cancel());
    progress?.animate([{ width: "0%" }, { width: "100%" }], { duration: DEMO_MS, easing: "linear", fill: "forwards" });
    try {
      // Drop
      setState("empty");
      choose(null);
      lightStep(1);
      await sleep(700, id);
      show(pointer, true);
      show(drag, true);
      win.dataset.pulse = "";
      await Promise.all([move(drag, [760, 700], [330, 262], 1200), move(pointer, [790, 730], [470, 300], 1200)]);
      await sleep(250, id);
      delete win.dataset.pulse;
      show(drag, false);
      // Pick a version
      const summary = $("[data-aw-summary]");
      const done = summary.innerHTML;
      summary.className = "aw-summary";
      summary.textContent = say("queue.summary.checking");
      $("[data-aw-report]").style.opacity = "0";
      setState("list");
      lightStep(2);
      await sleep(600, id);
      summary.className = "aw-summary warn";
      summary.innerHTML = done;
      await move(pointer, [470, 300], [470, 522], 900);
      await sleep(150, id);
      choose(24);
      await sleep(250, id);
      $("[data-aw-report]").style.opacity = "1";
      await sleep(1900, id);
      // Convert
      await move(pointer, [470, 522], [770, 596], 800);
      lightStep(3);
      const convert = $("[data-aw-convert]");
      convert.classList.add("press");
      await sleep(180, id);
      convert.classList.remove("press");
      setState("converting");
      show(pointer, false);
      const fill = $("[data-aw-fill]");
      const pct = $("[data-aw-pct]");
      const started = performance.now();
      await new Promise<void>((resolve) => {
        const tick = (now: number) => {
          if (id !== run) return resolve();
          const p = Math.min(1, (now - started) / 1300);
          fill.style.width = `${p * 100}%`;
          pct.textContent = `${Math.round(p * 100)}%`;
          if (p < 1) requestAnimationFrame(tick);
          else resolve();
        };
        requestAnimationFrame(tick);
      });
      await sleep(250, id);
      setState("done");
      await sleep(2200, id);
      // Your turn
      setState("empty");
      lightStep(1);
      setMode("turn");
      replay.hidden = false;
    } catch {
      // Stopped by a file of the visitor's own.
    }
  }

  // 3. The visitor's own file.
  let current: { file: File; found: Inspection } | null = null;
  const esc = (s: string) => s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);
  const icons = {
    alert: '<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/><path d="M12 9v4"/><path d="M12 17h.01"/>',
    check: '<path d="M20 6 9 17l-5-5"/>',
    flask:
      '<path d="M14 2v6a2 2 0 0 0 .245.96l5.51 10.08A2 2 0 0 1 18 22H6a2 2 0 0 1-1.755-2.96l5.51-10.08A2 2 0 0 0 10 8V2"/><path d="M6.453 15h11.094"/><path d="M8.5 2h7"/>',
    info: '<circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/>',
    ban: '<circle cx="12" cy="12" r="10"/><path d="m4.9 4.9 14.2 14.2"/>',
    minus: '<circle cx="12" cy="12" r="10"/><path d="M8 12h8"/>',
    puzzle:
      '<path d="M15.39 4.39a1 1 0 0 0 1.68-.474 2.5 2.5 0 1 1 3.014 3.015 1 1 0 0 0-.474 1.68l1.683 1.682a2.414 2.414 0 0 1 0 3.414L19.61 15.39a1 1 0 0 1-1.68-.474 2.5 2.5 0 1 0-3.014 3.015 1 1 0 0 1 .474 1.68l-1.683 1.682a2.414 2.414 0 0 1-3.414 0L8.61 19.61a1 1 0 0 0-1.68.474 2.5 2.5 0 1 1-3.014-3.015 1 1 0 0 0 .474-1.68l-1.683-1.682a2.414 2.414 0 0 1 0-3.414L4.39 8.61a1 1 0 0 1 1.68.474 2.5 2.5 0 1 0 3.014-3.015 1 1 0 0 1-.474-1.68l1.683-1.682a2.414 2.414 0 0 1 3.414 0z"/>',
    lock: '<rect width="18" height="11" x="3" y="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>',
    download: '<path d="M12 15V3"/><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="m7 10 5 5 5-5"/>',
  };
  const svg = (name: keyof typeof icons, size: number) =>
    `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${icons[name]}</svg>`;

  const describe = (file: File, found: Inspection) => {
    const ext = (file.name.match(/\.[^.]+$/)?.[0] ?? "?").toLowerCase();
    const size = `${(file.size / 1048576).toFixed(1)} MB`;
    let meta: string;
    if (found.problem) meta = esc(say(`problem.${found.problem}`));
    else if (found.version) {
      const v = found.version;
      const name = esc(say("file.version", { name: v.name ?? String(v.major) }));
      const number = `${v.major}.${v.minor}${v.patch ? `.${v.patch}` : ""}${v.release ? "" : ` ${esc(say("file.beta"))}`}`;
      meta = `${name} <em>/</em> <code>${number}</code> <em>/</em> <code class="dim">${size}</code>`;
    } else if (found.format) {
      meta = `${esc(say("file.format", { major: found.format.major, minor: found.format.minor }))} <em>/</em> <code class="dim">${size}</code>`;
    } else meta = "";
    return `<span class="aw-ext">${esc(ext)}</span><div class="aw-file"><b>${esc(file.name)}</b><span>${meta}</span></div><span class="aw-summary" data-aw-summary>${esc(say("queue.summary.checking"))}</span>`;
  };

  const renderReport = (report: Report, major: number) => {
    const target = tiles.find((t) => Number(t.dataset.awTile) === major);
    const verified = target?.querySelector("small")?.classList.contains("ok");
    const year = target?.querySelector("b")?.textContent ?? String(2000 + major);
    const counted = report.notes.filter((n) => n.code !== "experimental-target");
    const summary = $("[data-aw-summary]");
    if (report.blocked) {
      summary.className = "aw-summary bad";
      summary.innerHTML = `${svg("ban", 17)}${esc(say("state.blocked"))}`;
    } else if (counted.length === 0) {
      summary.className = "aw-summary ok";
      summary.innerHTML = `${svg("check", 17)}${esc(say("queue.summary.clean"))}`;
    } else {
      summary.className = "aw-summary warn";
      summary.innerHTML = `${svg("alert", 17)}${esc(say("queue.summary.changes", { count: counted.length }))}`;
    }
    const notes = report.notes.length
      ? report.notes
      : [{ severity: "ok", code: "no-changes", params: { year } } as Report["notes"][number]];
    const items = notes
      .map((n) => {
        const params: Record<string, string | number> = { year, ...n.params } as Record<string, string | number>;
        if (Array.isArray(n.params.names)) params.names = (n.params.names as string[]).join(", ");
        const hint = pick(`note.${n.code}-hint`);
        // The app's icons: a puzzle for effects and plugins, a minus for what is removed.
        const effects = n.code === "effects-removed" || n.code === "plugins-kept";
        const icon =
          n.severity === "block" ? svg("ban", 18)
          : effects ? svg("puzzle", 18)
          : n.severity === "warn" ? svg("alert", 18)
          : n.severity === "ok" ? svg("check", 18)
          : svg("minus", 18);
        const cls = n.code === "plugins-kept" ? "plain" : n.severity;
        return `<li class="${cls}">${icon}<span><b>${esc(say(`note.${n.code}`, params))}</b>${hint ? `<small>${esc(hint)}</small>` : ""}</span></li>`;
      })
      .join("");
    const badge = verified
      ? `<span class="aw-badge ok">${svg("check", 14)}${esc(say("target.verified"))}</span>`
      : `<span class="aw-badge exp">${svg("flask", 14)}${esc(say("target.experimental"))}</span>`;
    const box = $("[data-aw-report]");
    box.innerHTML = `<p class="aw-report-for"><span>${esc(say("report.for"))} After Effects ${esc(year)}</span>${badge}</p><ul>${items}</ul>`;
    box.style.opacity = "1";
    announce(box);
  };
  // The summary, then the report as plain sentences, for the live region.
  const announce = (box: HTMLElement | null) => {
    if (!live || !current) return;
    const parts = [current.file.name, $("[data-aw-summary]").textContent ?? ""];
    if (box) for (const el of box.querySelectorAll(".aw-report-for > span, li b, li small, .aw-badge")) parts.push(el.textContent ?? "");
    live.textContent = parts
      .map((p) => p.trim().replace(/[.:]$/, ""))
      .filter(Boolean)
      .join(". ")
      .concat(".");
  };

  const pickTarget = async (major: number) => {
    if (!current) return;
    choose(major);
    const box = $("[data-aw-report]");
    box.style.opacity = "0.4";
    const summary = $("[data-aw-summary]");
    summary.className = "aw-summary";
    summary.textContent = say("queue.summary.checking");
    const file = current.file;
    if (file.size > REPORT_LIMIT) return noReport(major, site.bigFile, site.bigFileSummary);
    try {
      const report = await reportFile(file, major);
      if (current?.file === file) renderReport(report, major);
    } catch {
      if (current?.file === file) noReport(major, site.reportFailed, site.bigFileSummary);
    }
  };

  async function own(file: File | undefined) {
    if (!file) return;
    run++; // stops the demo
    replay.hidden = false;
    setMode("own");
    progress?.getAnimations().forEach((a) => a.cancel());
    for (const el of [pointer, drag]) el.getAnimations().forEach((a) => a.cancel());
    lightStep(2);
    let found: Inspection;
    try {
      found = await inspectFile(file);
    } catch {
      found = { kind: "unknown", problem: "unreadable", version: null, format: null, release: null, targets: [] };
    }
    current = { file, found };
    $("[data-aw-row]").innerHTML = describe(file, found);
    $("[data-aw-save]").innerHTML = `${svg("lock", 16)}${esc(site.neverLeft)}`;
    const convert = $("[data-aw-convert]");
    const link = document.createElement("a");
    link.className = "aw-convert";
    link.dataset.awConvert = "";
    link.href = document.querySelector<HTMLAnchorElement>("[data-os-download]")?.href ?? "#download";
    link.innerHTML = `${svg("download", 22)}${esc(site.downloadToConvert)}`;
    convert.replaceWith(link);
    const majors = new Set(found.targets.map((t) => t.major));
    tiles.forEach((t) => t.classList.toggle("off", !majors.has(Number(t.dataset.awTile))));
    tileRow.classList.toggle("pickable", majors.size > 0);
    setState("list");
    const box = $("[data-aw-report]");
    if (found.problem || majors.size === 0) {
      choose(null);
      const summary = $("[data-aw-summary]");
      summary.className = found.problem ? "aw-summary bad" : "aw-summary";
      summary.innerHTML = found.problem ? `${svg("ban", 17)}${esc(say("state.blocked"))}` : esc(say("state.nothingToDo"));
      box.innerHTML = "";
      box.style.opacity = "0";
      announce(null);
      return;
    }
    const verified = found.targets.find((t) => t.status === "verified");
    await pickTarget((verified ?? found.targets[0]).major);
  }

  // A report the page could not make: said plainly, never as "can't convert".
  const noReport = (major: number, text: string, summaryText: string) => {
    choose(major);
    const summary = $("[data-aw-summary]");
    summary.className = "aw-summary";
    summary.textContent = summaryText;
    const box = $("[data-aw-report]");
    box.innerHTML = `<ul><li class="plain">${svg("info", 18)}<span><b>${esc(text)}</b></span></li></ul>`;
    box.style.opacity = "1";
    announce(box);
  };

  tileRow.addEventListener("click", (e) => {
    const tile = (e.target as Element).closest<HTMLElement>("[data-aw-tile]");
    if (current && tile && !tile.classList.contains("off")) pickTarget(Number(tile.dataset.awTile));
  });
  tileRow.addEventListener("keydown", (e) => {
    const tile = (e.target as Element).closest<HTMLElement>("[data-aw-tile]");
    if (!current || !tile) return;
    const usable = tiles.filter((t) => !t.classList.contains("off"));
    const step = e.key === "ArrowRight" || e.key === "ArrowDown" ? 1 : e.key === "ArrowLeft" || e.key === "ArrowUp" ? -1 : 0;
    if (step) {
      e.preventDefault();
      const next = usable[(usable.indexOf(tile) + step + usable.length) % usable.length];
      if (!next) return;
      pickTarget(Number(next.dataset.awTile));
      next.focus();
    } else if (e.key === " " || e.key === "Enter") {
      e.preventDefault();
      if (!tile.classList.contains("off")) pickTarget(Number(tile.dataset.awTile));
    }
  });
  input.addEventListener("change", () => own(input.files?.[0]));
  // The whole window takes a file: click it (or press Enter on it) to choose one, any time but on
  // your own file's controls.
  frame.addEventListener("click", (e) => {
    if (stage.dataset.mode === "own" && !(e.target as Element).closest(".aw-ghostbtn")) return;
    input.click();
  });
  frame.addEventListener("keydown", (e) => {
    if (e.target === frame && (e.key === "Enter" || e.key === " ")) {
      e.preventDefault();
      input.click();
    }
  });
  for (const button of document.querySelectorAll("[data-aw-try]")) {
    button.addEventListener("click", (e) => {
      e.preventDefault();
      input.click();
    });
  }
  stage.addEventListener("dragover", (e) => {
    e.preventDefault();
    win.classList.add("over");
  });
  stage.addEventListener("dragleave", (e) => {
    if (!stage.contains(e.relatedTarget as Node)) win.classList.remove("over");
  });
  stage.addEventListener("drop", (e) => {
    e.preventDefault();
    win.classList.remove("over");
    own(e.dataTransfer?.files[0]);
  });
  window.addEventListener("dragover", (e) => e.preventDefault());
  window.addEventListener("drop", (e) => e.preventDefault());
  replay.addEventListener("click", () => demo());

  // Play once the window is on screen; with reduced motion, stay on the report.
  if (reduced) {
    lightStep(1);
    setState("empty");
    setMode("turn");
    replay.hidden = true;
    return;
  }
  const io = new IntersectionObserver((entries) => {
    if (entries.some((e) => e.isIntersecting)) {
      io.disconnect();
      demo();
    }
  });
  io.observe(frame);
}
