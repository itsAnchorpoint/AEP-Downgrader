// Development only (imported behind import.meta.env.DEV): puts the app into a named state from the
// URL, so every screen can be captured in a browser with the mock backend.
//
//   http://localhost:1420/?demo=queue&theme=dark&lang=en&accent=lime
//
// States: empty, queue, report, converting, done, settings, update, installing, installed, whatsnew,
// history, log, stopped. Without ?demo the app starts normally.
import { initialState, useAppStore } from "../store";
import { ACCENTS, THEMES, type Accent, type Theme } from "./settings";

const PATHS = ["/demo/Intro_v05.aep", "/demo/Lower_thirds.aep", "/demo/Glow_pack.ffx", "/demo/._Intro_v05.aep"];

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

async function ready() {
  for (let i = 0; i < 100 && useAppStore.getState().targets.length === 0; i++) await wait(20);
}

async function queue() {
  const store = useAppStore.getState();
  await store.addPaths(PATHS);
  await useAppStore.getState().setTarget(24);
}

/** Marks the first `done` convertible files as converted and the next one as running. */
function progress(done: number) {
  const { files, reports } = useAppStore.getState();
  const ready = files.filter((f) => !f.problem && !reports[f.id]?.blocked);
  useAppStore.setState({ phase: "converting", jobIds: ready.map((f) => f.id), progress: {} });
  ready.forEach((f, i) => {
    useAppStore
      .getState()
      .handleProgress(
        i < done
          ? { id: f.id, state: "done", outputPath: f.path.replace(/(\.\w+)$/, "_AE2024$1") }
          : { id: f.id, state: i === done ? "running" : "queued" },
      );
  });
}

export async function runDemo() {
  const params = new URLSearchParams(window.location.search);
  const state = params.get("demo");
  if (!state) return;
  await ready();
  await wait(700); // let the start-up update check of the mock finish
  const store = useAppStore.getState();
  const theme = params.get("theme") as Theme | null;
  const accent = params.get("accent") as Accent | null;
  const lang = params.get("lang");
  store.updateSettings({
    ...(theme && THEMES.includes(theme) ? { theme } : {}),
    ...(accent && ACCENTS.includes(accent) ? { accent } : {}),
    ...(lang === "en" || lang === "ru" ? { language: lang } : {}),
  });
  // The mock offers an update on every start; only the update states keep it.
  if (!["update", "installing", "installed", "whatsnew"].includes(state)) {
    useAppStore.setState({ update: initialState.update });
  }

  switch (state) {
    case "queue":
      await queue();
      break;
    case "report":
      await queue();
      useAppStore.getState().toggleExpanded(useAppStore.getState().files[0].id);
      break;
    case "converting":
      await queue();
      progress(1);
      break;
    case "done":
      await queue();
      progress(99);
      break;
    case "stopped": {
      await queue();
      progress(1);
      const { jobIds } = useAppStore.getState();
      jobIds.slice(1).forEach((id) => useAppStore.getState().handleProgress({ id, state: "cancelled" }));
      break;
    }
    case "settings":
      store.setView("settings");
      break;
    case "update":
      await store.checkUpdate();
      break;
    case "installing":
      await store.checkUpdate();
      useAppStore.setState((s) => ({ update: { ...s.update, status: "downloading", progress: 0.42 } }));
      break;
    case "installed":
      await store.checkUpdate();
      useAppStore.setState((s) => ({ update: { ...s.update, status: "installed", progress: 1 } }));
      break;
    case "whatsnew":
      await store.checkUpdate();
      store.setWhatsNewOpen(true, "update");
      break;
    case "history":
      store.setWhatsNewOpen(true, "history");
      break;
    case "log":
      await queue();
      store.setLogOpen(true);
      break;
  }
  document.documentElement.dataset.demoReady = state;
}
