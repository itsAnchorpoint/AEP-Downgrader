import { motion, MotionConfig } from "motion/react";
import { useEffect } from "react";
import { Converting } from "./components/Converting";
import { Done } from "./components/Done";
import { DropZone } from "./components/DropZone";
import { Header } from "./components/Header";
import { LogPanel } from "./components/LogPanel";
import { Queue } from "./components/Queue";
import { Settings } from "./components/Settings";
import { UpdateBanner } from "./components/UpdateBanner";
import { WhatsNew } from "./components/WhatsNew";
import { ipc } from "./lib/ipc";
import { isConvertible, useAppStore } from "./store";

/** Keeps <html data-theme> in sync with the setting and, for "system", with the OS. */
function useTheme() {
  const theme = useAppStore((s) => s.settings.theme);
  useEffect(() => {
    const query = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => {
      const dark = theme === "dark" || (theme === "system" && query.matches);
      document.documentElement.dataset.theme = theme === "ae" ? "ae" : dark ? "dark" : "light";
    };
    apply();
    query.addEventListener("change", apply);
    return () => query.removeEventListener("change", apply);
  }, [theme]);
}

function useAccent() {
  const accent = useAppStore((s) => s.settings.accent);
  useEffect(() => {
    document.documentElement.dataset.accent = accent;
  }, [accent]);
}

function useLanguageAttribute() {
  const language = useAppStore((s) => s.settings.language);
  useEffect(() => {
    document.documentElement.lang = language;
  }, [language]);
}

function Screen() {
  const view = useAppStore((s) => s.view);
  const phase = useAppStore((s) => s.phase);
  const hasFiles = useAppStore((s) => s.files.length > 0);
  if (view === "settings") return <Settings />;
  if (phase === "converting") return <Converting />;
  if (phase === "done") return <Done />;
  return hasFiles ? <Queue /> : <DropZone />;
}

/** Remounts on every screen change so each screen fades in. */
function useScreenKey() {
  const view = useAppStore((s) => s.view);
  const phase = useAppStore((s) => s.phase);
  const hasFiles = useAppStore((s) => s.files.length > 0);
  return view === "settings" ? "settings" : phase === "queue" ? (hasFiles ? "queue" : "empty") : phase;
}

/**
 * Keyboard: Ctrl/Cmd+O adds files, Ctrl/Cmd+Enter converts the queue, Esc leaves Settings.
 * Keys typed into a field stay with the field (except Esc).
 */
function useShortcuts() {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const typing = e.target instanceof HTMLElement && e.target.closest("input, textarea, [contenteditable]");
      const mod = e.ctrlKey || e.metaKey;
      const { view, phase, files, reports, browse, convert, setView, whatsNewOpen } = useAppStore.getState();
      if (e.key === "Escape" && view === "settings" && !whatsNewOpen) {
        e.preventDefault();
        setView("main");
      } else if (!typing && mod && e.key.toLowerCase() === "o" && phase === "queue") {
        e.preventDefault();
        void browse();
      } else if (!typing && mod && e.key === "Enter" && phase === "queue" && view === "main") {
        if (files.some((f) => isConvertible(f, reports))) {
          e.preventDefault();
          void convert();
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
}

export default function App() {
  const init = useAppStore((s) => s.init);
  const addPaths = useAppStore((s) => s.addPaths);
  const setDragging = useAppStore((s) => s.setDragging);
  useTheme();
  useAccent();
  const screenKey = useScreenKey();
  useLanguageAttribute();
  useShortcuts();

  useEffect(() => {
    void init();
    const unlisten = ipc.onFileDrop({ hover: setDragging, drop: (paths) => void addPaths(paths) });
    return () => void unlisten.then((fn) => fn());
  }, [init, addPaths, setDragging]);

  return (
    <MotionConfig reducedMotion="user">
      <div className="relative flex h-full flex-col">
        <Header />
        <UpdateBanner />
        <main className="min-h-0 flex-1 px-6 pt-2 pb-5 max-[820px]:px-4">
          <motion.div
            key={screenKey}
            initial={{ opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.25, ease: [0.2, 0.9, 0.2, 1] }}
            className="h-full"
          >
            <Screen />
          </motion.div>
        </main>
        <LogPanel />
      </div>
      <WhatsNew />
    </MotionConfig>
  );
}
