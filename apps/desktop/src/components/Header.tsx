import { ScrollText, SlidersHorizontal } from "lucide-react";
import { useTranslation } from "react-i18next";
import { version } from "../../package.json";
import { useAppStore } from "../store";
import { IconButton } from "./ui";

/** The Fold logo's small drawing; the band follows the accent colour. */
function LogoMark() {
  return (
    <svg viewBox="0 0 32 32" className="size-7 shrink-0" aria-hidden>
      <path
        d="M19.8 1C23.7 1 25.7 1 27.2 1.8C28.5 2.4 29.6 3.5 30.2 4.8C31 6.3 31 8.3 31 12.2V19.8C31 23.7 31 25.7 30.2 27.2C29.6 28.5 28.5 29.6 27.2 30.2C25.7 31 23.7 31 19.8 31H12.2C8.3 31 6.3 31 4.8 30.2C3.5 29.6 2.4 28.5 1.8 27.2C1 25.7 1 23.7 1 19.8V12.2C1 8.3 1 6.3 1.8 4.8C2.4 3.5 3.5 2.4 4.8 1.8C6.3 1 8.3 1 12.2 1Z"
        fill="#0b0b0c"
      />
      <path d="M11 12.7V19.9L1 13.3V6.1L11 12.7Z" fill="var(--color-accent)" />
      <path d="M11 19.9V12.7L16 9.4V16.6L11 19.9Z" fill="color-mix(in srgb, var(--color-accent) 68%, black)" />
      <path
        d="M16 16.6V9.4L20.9 12.6L18.9 9.3L22.1 8.2L31 22.9L22.1 25.9L18.9 20.6L20.9 19.9L16 16.6Z"
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

  return (
    <header className="flex h-12 shrink-0 items-center justify-between gap-3 px-5">
      <div className="flex items-center gap-2.5">
        <LogoMark />
        <span className="text-base font-extrabold tracking-tight">{t("app.name")}</span>
        <span className="rounded-[5px] bg-accent px-1.5 font-mono text-xs font-bold text-accent-fg">{version}</span>
      </div>
      <div className="flex items-center gap-0.5">
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
