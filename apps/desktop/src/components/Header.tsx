import { Activity, ScrollText, SlidersHorizontal } from "lucide-react";
import { useTranslation } from "react-i18next";
import { version } from "../../package.json";
import { useAppStore } from "../store";
import { LOG_PANEL_ID, LOG_TOGGLE_ID } from "./LogPanel";
import { IconButton } from "./ui";

/** The Trail logo's small drawing; the arrow follows the accent colour. */
function LogoMark() {
  return (
    <svg viewBox="0 0 32 32" className="size-7 shrink-0" aria-hidden>
      <path
        d="M19.8 1C23.7 1 25.7 1 27.2 1.8C28.5 2.4 29.6 3.5 30.2 4.8C31 6.3 31 8.3 31 12.2V19.8C31 23.7 31 25.7 30.2 27.2C29.6 28.5 28.5 29.6 27.2 30.2C25.7 31 23.7 31 19.8 31H12.2C8.3 31 6.3 31 4.8 30.2C3.5 29.6 2.4 28.5 1.8 27.2C1 25.7 1 23.7 1 19.8V12.2C1 8.3 1 6.3 1.8 4.8C2.4 3.5 3.5 2.4 4.8 1.8C6.3 1 8.3 1 12.2 1Z"
        fill="#0b0b0c"
      />
      <path
        d="M8.5 6.1C10.4 6.1 11.9 7.6 11.9 9.5C11.9 9.6 11.9 9.8 11.9 9.9L14 12L17 9L22.4 14.4V12.3H26.6V21.6H17.3V17.4H19.4L17 15L14 18L8.9 12.9C8.8 12.9 8.6 12.9 8.5 12.9C6.6 12.9 5.1 11.4 5.1 9.5C5.1 7.6 6.6 6.1 8.5 6.1Z"
        fill="var(--color-accent)"
      />
    </svg>
  );
}

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
    <header className="flex h-12 shrink-0 items-center justify-between gap-3 px-5">
      <div className="flex items-center gap-2.5">
        <LogoMark />
        <span className="text-base font-extrabold tracking-tight">{t("app.name")}</span>
        <span className="rounded-[5px] bg-accent px-1.5 font-mono text-xs font-bold text-accent-fg">{version}</span>
      </div>
      <div className="flex items-center gap-0.5">
        <IconButton
          id={LOG_TOGGLE_ID}
          label={errors > 0 ? t("header.logErrors", { count: errors }) : t("header.log")}
          aria-expanded={logOpen}
          aria-controls={LOG_PANEL_ID}
          onClick={() => setLogOpen(!logOpen)}
          className={`relative ${logOpen ? "bg-press text-fg" : ""}`}
        >
          <Activity className="size-4.5" aria-hidden />
          {errors > 0 && (
            <span className="absolute top-1.5 right-1.5 size-2 rounded-full bg-danger" aria-hidden />
          )}
        </IconButton>
        <IconButton label={t("header.whatsNew")} onClick={() => setWhatsNewOpen(true)} className="relative">
          <ScrollText className="size-4.5" aria-hidden />
          {update.status === "available" && (
            <span className="absolute top-1.5 right-1.5 size-2 rounded-full bg-accent" aria-hidden />
          )}
        </IconButton>
        <IconButton
          label={t("header.settings")}
          aria-pressed={inSettings}
          onClick={() => setView(inSettings ? "main" : "settings")}
          className={inSettings ? "bg-press text-fg" : ""}
        >
          <SlidersHorizontal className="size-4.5" aria-hidden />
        </IconButton>
      </div>
    </header>
  );
}
