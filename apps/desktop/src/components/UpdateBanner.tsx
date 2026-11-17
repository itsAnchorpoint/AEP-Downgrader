import { ArrowUpToLine, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../store";
import { Button, IconButton } from "./ui";

export function UpdateBanner() {
  const { t } = useTranslation();
  const update = useAppStore((s) => s.update);
  const setWhatsNewOpen = useAppStore((s) => s.setWhatsNewOpen);
  const installUpdate = useAppStore((s) => s.installUpdate);
  const restartApp = useAppStore((s) => s.restartApp);
  const dismiss = useAppStore((s) => s.dismissUpdateBanner);
  const visible = update.status !== "none" && !update.bannerDismissed;
  const busy = update.status === "downloading" || update.status === "installing";
  const percent = update.progress === null ? null : Math.round(update.progress * 100);
  const manual = update.installKind === "manual";

  let text = "";
  if (update.status === "available") text = t("update.title", { version: update.version });
  else if (update.status === "downloading")
    text = percent === null ? t("update.downloading") : t("update.downloadingPercent", { percent });
  else if (update.status === "installing") text = t("update.installing");
  else if (update.status === "installed") text = t("update.installed");
  else if (update.status === "failed") text = t("update.failed");

  const primary = "min-h-8! bg-accent-fg! text-accent!";

  return (
    <AnimatePresence>
      {visible && (
        <motion.div
          role="status"
          initial={{ opacity: 0, y: -16 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.35, ease: [0.2, 0.8, 0.2, 1] }}
          className="relative flex shrink-0 items-center gap-3 bg-accent py-2 pr-2.5 pl-6 text-sm font-bold text-accent-fg [--color-focus:var(--color-accent-fg)]"
        >
          <ArrowUpToLine className="size-5 shrink-0" strokeWidth={2.75} aria-hidden />
          <span className="min-w-0 flex-1 truncate">{text}</span>
          {update.status === "available" && (
            <>
              <Button variant="link" className="text-accent-fg" onClick={() => setWhatsNewOpen(true, "update")}>
                {t("update.whatsNew")}
              </Button>
              <Button variant="primary" className={primary} onClick={() => void installUpdate()}>
                {manual ? t("update.download") : t("update.action")}
              </Button>
            </>
          )}
          {update.status === "installed" && (
            <Button variant="primary" className={primary} onClick={() => void restartApp()}>
              {t("update.restart")}
            </Button>
          )}
          {update.status === "failed" && (
            <Button variant="primary" className={primary} onClick={() => void installUpdate()}>
              {t("update.download")}
            </Button>
          )}
          {!busy && (
            <IconButton
              label={t("update.dismiss")}
              onClick={dismiss}
              className="size-8! text-accent-fg! hover:bg-black/15!"
            >
              <X className="size-4" aria-hidden />
            </IconButton>
          )}
          {busy && (
            <span
              className="absolute bottom-0 left-0 h-0.75 bg-accent-fg transition-[width] duration-300"
              style={{ width: `${percent ?? 0}%` }}
              aria-hidden
            />
          )}
        </motion.div>
      )}
    </AnimatePresence>
  );
}
