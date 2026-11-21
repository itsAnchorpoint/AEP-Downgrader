import { ArrowDownToLine, X } from "lucide-react";
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

  const primary = "min-h-8!";

  return (
    <AnimatePresence>
      {visible && (
        <motion.div
          initial={{ opacity: 0, y: -16 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.35, ease: [0.2, 0.8, 0.2, 1] }}
          className="relative mx-6 mt-1 mb-2 flex min-h-11 shrink-0 items-center gap-3 rounded-full surface py-1.5 pr-1.5 pl-4 text-sm font-bold max-[820px]:mx-4"
        >
          <ArrowDownToLine className="size-5 shrink-0 text-accent-text" strokeWidth={2.75} aria-hidden />
          <span role="status" className="min-w-0 flex-1 truncate">
            {text}
          </span>
          {busy && (
            <span className="mr-3 h-1.5 w-32 shrink-0 overflow-hidden rounded-full bg-raised" aria-hidden>
              <span
                className="block h-full rounded-full bg-accent-text transition-[width] duration-300"
                style={{ width: `${percent ?? 0}%` }}
              />
            </span>
          )}
          {update.status === "available" && (
            <>
              <Button variant="link" onClick={() => setWhatsNewOpen(true, "update")}>
                {t("update.whatsNew")}
              </Button>
              <Button className={primary} onClick={() => void installUpdate()}>
                {manual ? t("update.download") : t("update.action")}
              </Button>
            </>
          )}
          {update.status === "installed" && (
            <Button className={primary} onClick={() => void restartApp()}>
              {t("update.restart")}
            </Button>
          )}
          {update.status === "failed" && (
            <Button className={primary} onClick={() => void installUpdate()}>
              {t("update.download")}
            </Button>
          )}
          {!busy && (
            <IconButton
              label={t("update.dismiss")}
              onClick={dismiss}
              className="size-8! rounded-full!"
            >
              <X className="size-4" aria-hidden />
            </IconButton>
          )}
        </motion.div>
      )}
    </AnimatePresence>
  );
}
