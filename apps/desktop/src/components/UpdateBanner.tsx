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
  const dismiss = useAppStore((s) => s.dismissUpdateBanner);
  const visible = update.status !== "none" && !update.bannerDismissed;

  return (
    <AnimatePresence>
      {visible && (
        <motion.div
          role="status"
          initial={{ opacity: 0, y: -16 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.35, ease: [0.2, 0.8, 0.2, 1] }}
          className="flex shrink-0 items-center gap-3 bg-accent py-2 pr-2.5 pl-6 text-sm font-bold text-accent-fg [--color-focus:var(--color-accent-fg)]"
        >
          <ArrowUpToLine className="size-5 shrink-0" strokeWidth={2.75} aria-hidden />
          <span className="min-w-0 flex-1 truncate">
            {update.status === "installing" && t("update.installing")}
            {update.status === "installed" && t("update.installed")}
            {update.status === "available" && t("update.title", { version: update.version })}
          </span>
          {update.status === "available" && (
            <>
              <Button variant="link" className="text-accent-fg" onClick={() => setWhatsNewOpen(true)}>
                {t("update.whatsNew")}
              </Button>
              <Button
                variant="primary"
                className="min-h-8! bg-accent-fg! text-accent!"
                onClick={() => void installUpdate()}
              >
                {t("update.action")}
              </Button>
            </>
          )}
          <IconButton label={t("update.dismiss")} onClick={dismiss} className="size-8! text-accent-fg! hover:bg-black/15!">
            <X className="size-4" aria-hidden />
          </IconButton>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
