import { ChevronsDown, ScrollText, SlidersHorizontal } from "lucide-react";
import { useTranslation } from "react-i18next";
import { version } from "../../package.json";
import { useAppStore } from "../store";
import { IconButton } from "./ui";

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
        <span className="grid size-6 place-items-center rounded-[7px] bg-accent text-accent-fg">
          <ChevronsDown className="size-4" strokeWidth={3} aria-hidden />
        </span>
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
