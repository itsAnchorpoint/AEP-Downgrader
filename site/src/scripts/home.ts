// The home page's behavior: the download for this system, the version picker, the first-launch tabs
// and the download choices (the live window has its own script, app-window.ts). Everything works without it too: links lead to
// the download section and the server-rendered defaults stay in place.

type Status = "verified" | "experimental";
interface PageData {
  lang: string;
  downloads: Record<string, string>;
  os: Record<OS, string>;
  heroDownloadFor: string;
  targets: { year: number; major: number; status: Status; basis: string }[];
  versions: Record<string, string>;
}
type OS = "windows" | "macos" | "linux";

const data: PageData = JSON.parse(document.getElementById("page-data")?.textContent ?? "{}");
const fmt = (s: string, vars: Record<string, string | number>) => s.replace(/\{(\w+)\}/g, (_, k) => String(vars[k] ?? ""));

function detectOS(): OS | null {
  const ua = navigator.userAgent;
  const platform = (navigator as Navigator & { userAgentData?: { platform?: string } }).userAgentData?.platform ?? navigator.platform ?? "";
  if (/android|iphone|ipad|ipod/i.test(ua)) return null;
  if (/win/i.test(platform) || /windows/i.test(ua)) return "windows";
  if (/mac/i.test(platform) || /mac os x/i.test(ua)) return "macos";
  if (/linux|x11/i.test(platform) || /linux|x11/i.test(ua)) return "linux";
  return null;
}
const fileFor: Record<OS, string> = {
  windows: data.downloads.windowsSetup,
  macos: data.downloads.macos,
  linux: data.downloads.appimage,
};
const os = detectOS();

// The download for this system: the hero button, the row in the download section, the first-launch tab.
if (os) {
  for (const button of document.querySelectorAll<HTMLAnchorElement>("[data-os-download]")) {
    button.href = fileFor[os];
    const label = button.querySelector("[data-os-label]");
    if (label) label.textContent = fmt(data.heroDownloadFor, { os: data.os[os] });
  }
  const row = document.querySelector(`[data-os-row="${os}"]`);
  if (row) {
    row.classList.add("current");
    const button = row.querySelector(".dl-btn");
    button?.classList.replace("btn-secondary", "btn-primary");
    row.parentElement?.prepend(row);
  }
}

// Radio groups made of buttons: click and arrow keys move the choice.
function radioGroup(group: Element, onChange: (button: HTMLButtonElement) => void) {
  const buttons = [...group.querySelectorAll<HTMLButtonElement>('[role="radio"]')];
  const select = (button: HTMLButtonElement, focus = false) => {
    for (const b of buttons) {
      b.setAttribute("aria-checked", String(b === button));
      b.tabIndex = b === button ? 0 : -1;
    }
    if (focus) button.focus();
    onChange(button);
  };
  for (const b of buttons) {
    b.tabIndex = b.getAttribute("aria-checked") === "true" ? 0 : -1;
    b.addEventListener("click", () => select(b));
    b.addEventListener("keydown", (e) => {
      const step = e.key === "ArrowRight" || e.key === "ArrowDown" ? 1 : e.key === "ArrowLeft" || e.key === "ArrowUp" ? -1 : 0;
      if (!step) return;
      e.preventDefault();
      select(buttons[(buttons.indexOf(b) + step + buttons.length) % buttons.length], true);
    });
  }
  return (match: (b: HTMLButtonElement) => boolean) => {
    const button = buttons.find(match);
    if (button) select(button);
  };
}

// Which After Effects do you have?
const picker = document.querySelector(".picker");
if (picker) {
  const years = [2023, 2024, 2025, 2026, 2027];
  const opens = document.querySelector("[data-opens]");
  const status = document.querySelector("[data-status]");
  const statusText = document.querySelector("[data-status-text]");
  const statusIcon = status?.querySelector("svg");
  const icons = {
    verified: '<path d="M20 6 9 17l-5-5"/>',
    experimental:
      '<path d="M14 2v6a2 2 0 0 0 .245.96l5.51 10.08A2 2 0 0 1 18 22H6a2 2 0 0 1-1.755-2.96l5.51-10.08A2 2 0 0 0 10 8V2"/><path d="M6.453 15h11.094"/><path d="M8.5 2h7"/>',
  };
  const list = (ys: number[]) =>
    ys.length <= 1 ? String(ys[0] ?? "") : `${ys.slice(0, -1).join(", ")} ${data.versions.and} ${ys[ys.length - 1]}`;
  const choose = radioGroup(picker, (button) => {
    const year = Number(button.dataset.year);
    const target = data.targets.find((t) => t.year === year);
    const state: Status = target?.status ?? "experimental";
    if (opens) opens.textContent = fmt(data.versions.opens, { list: list(years.filter((y) => y > year)), year });
    if (status) status.className = `status status-${state} versions-status`;
    if (statusIcon) statusIcon.innerHTML = icons[state];
    if (statusText) statusText.textContent = fmt(data.versions[`${state}Text`], { year });
    for (const li of document.querySelectorAll<HTMLElement>(".steps [data-leaves]")) {
      li.hidden = Number(li.dataset.leaves) - 1 < year - 2000;
    }
  });
  // A year on the hero's staircase picks it here.
  for (const link of document.querySelectorAll<HTMLAnchorElement>("a[data-year]")) {
    link.addEventListener("click", () => choose((b) => b.dataset.year === link.dataset.year));
  }
}

// First-launch tabs.
const tabs = document.querySelector("[data-tabs]");
if (tabs) {
  const buttons = [...tabs.querySelectorAll<HTMLButtonElement>('[role="tab"]')];
  const show = (button: HTMLButtonElement, focus = false) => {
    for (const b of buttons) {
      const on = b === button;
      b.setAttribute("aria-selected", String(on));
      b.tabIndex = on ? 0 : -1;
      const panel = document.getElementById(b.getAttribute("aria-controls") ?? "");
      if (panel) panel.hidden = !on;
    }
    if (focus) button.focus();
  };
  for (const b of buttons) {
    b.addEventListener("click", () => show(b));
    b.addEventListener("keydown", (e) => {
      const step = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0;
      if (!step) return;
      e.preventDefault();
      show(buttons[(buttons.indexOf(b) + step + buttons.length) % buttons.length], true);
    });
  }
  const mine = buttons.find((b) => b.dataset.os === os);
  if (mine) show(mine);
  else buttons.forEach((b, i) => (b.tabIndex = i === 0 ? 0 : -1));
}

// Download choices: installer or portable, AppImage or .deb.
for (const group of document.querySelectorAll(".dl-row .segmented")) {
  const row = group.closest(".dl-row");
  radioGroup(group, (button) => {
    const link = row?.querySelector<HTMLAnchorElement>("[data-href-out]");
    const text = row?.querySelector("[data-text-out]");
    if (link && button.dataset.href) link.href = button.dataset.href;
    if (text && button.dataset.text) text.textContent = button.dataset.text;
    if (os === "windows" && row?.getAttribute("data-os-row") === "windows") {
      for (const hero of document.querySelectorAll<HTMLAnchorElement>("[data-os-download]")) hero.href = button.dataset.href ?? hero.href;
    }
  });
}
