import { Activity, Copy, Minus, ScrollText, SlidersHorizontal, Square, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { isTauri } from "../lib/ipc-tauri";
import { platform } from "../lib/platform";
import { watchMaximized, windowAction } from "../lib/window";
import { useAppStore } from "../store";
import { LOG_PANEL_ID, LOG_TOGGLE_ID } from "./LogPanel";
import { IconButton } from "./ui";

/** A dot on a tool button; the ring cuts it out of the icon behind it. */
function Dot({ tone }: { tone: string }) {
  return <span className={`absolute top-1.5 right-1.5 size-2 rounded-full ring-2 ring-bg ${tone}`} aria-hidden />;
}

/** Minimize, maximize and close for the frameless window on Windows (46 px cells, red close). */
function WindowControls() {
  const { t } = useTranslation();
  const [maximized, setMaximized] = useState(false);
  useEffect(() => {
    const stop = watchMaximized(setMaximized);
    return () => void stop.then((fn) => fn());
  }, []);
  const cell =
    "grid h-full w-[46px] place-items-center text-muted transition-colors hover:bg-hover hover:text-fg active:bg-press";
  return (
    <div className="flex h-full items-stretch">
      <button type="button" aria-label={t("window.minimize")} className={cell} onClick={() => void windowAction("minimize")}>
        <Minus className="size-4" strokeWidth={1.5} aria-hidden />
      </button>
      <button
        type="button"
        aria-label={t(maximized ? "window.restore" : "window.maximize")}
        className={cell}
        onClick={() => void windowAction("toggleMaximize")}
      >
        {maximized ? (
          <Copy className="size-3.5 -scale-x-100" strokeWidth={1.5} aria-hidden />
        ) : (
          <Square className="size-3.5" strokeWidth={1.5} aria-hidden />
        )}
      </button>
      <button
        type="button"
        aria-label={t("window.close")}
        className={`${cell} hover:bg-[#c42b1c]! hover:text-white! active:bg-[#a52314]!`}
        onClick={() => void windowAction("close")}
      >
        <X className="size-4" strokeWidth={1.5} aria-hidden />
      </button>
    </div>
  );
}

/**
 * The title bar. On macOS and Windows it is the window's own (tauri.conf.json: overlay title bar;
 * tauri.windows.conf.json: no frame), so it drags the window and leaves room for the traffic
 * lights or draws the window buttons. On Linux the system draws the frame and this is a toolbar.
 * The product name lives on the start screen, so the bar carries only the tools.
 */
export function Header() {
  const { t } = useTranslation();
  const view = useAppStore((s) => s.view);
  const setView = useAppStore((s) => s.setView);
  const setWhatsNewOpen = useAppStore((s) => s.setWhatsNewOpen);
  const update = useAppStore((s) => s.update);
  const inSettings = view === "settings";
  const logOpen = useAppStore((s) => s.logOpen);
  const setLogOpen = useAppStore((s) => s.setLogOpen);
  const errors = useAppStore((s) => s.log.filter((e) => e.level === "error").length);

  return (
    <header data-tauri-drag-region className="relative flex h-11 shrink-0 items-center">
      {platform === "mac" &&
        (isTauri() ? (
          <span className="w-[78px] shrink-0" data-tauri-drag-region />
        ) : (
          // Browser preview only: where macOS draws the traffic lights.
          <span className="flex w-[78px] shrink-0 gap-2 pl-[18px]" aria-hidden>
            {["#ff5f57", "#febc2e", "#28c840"].map((c) => (
              <span key={c} className="size-3 rounded-full" style={{ background: c }} />
            ))}
          </span>
        ))}
      <span className="flex-1 self-stretch" data-tauri-drag-region />
      <div className={`flex items-center gap-0.5 ${platform === "windows" ? "pr-2" : "pr-4"}`}>
        <IconButton
          id={LOG_TOGGLE_ID}
          label={errors > 0 ? t("header.logErrors", { count: errors }) : t("header.log")}
          aria-expanded={logOpen}
          aria-controls={LOG_PANEL_ID}
          onClick={() => setLogOpen(!logOpen)}
          className={`relative ${logOpen ? "bg-press text-fg" : ""}`}
        >
          <Activity className="size-4.5" strokeWidth={2.25} aria-hidden />
          {errors > 0 && <Dot tone="bg-danger" />}
        </IconButton>
        <IconButton label={t("header.whatsNew")} onClick={() => setWhatsNewOpen(true)} className="relative">
          <ScrollText className="size-4.5" strokeWidth={2.25} aria-hidden />
          {update.status === "available" && <Dot tone="bg-accent" />}
        </IconButton>
        <IconButton
          label={t("header.settings")}
          aria-pressed={inSettings}
          onClick={() => setView(inSettings ? "main" : "settings")}
          className={inSettings ? "bg-press text-fg" : ""}
        >
          <SlidersHorizontal className="size-4.5" strokeWidth={2.25} aria-hidden />
        </IconButton>
      </div>
      {platform === "windows" && <WindowControls />}
    </header>
  );
}
