import { Check, Heart, Plus, X } from "lucide-react";
import { motion } from "motion/react";
import { useTranslation } from "react-i18next";
import { REPO_URL } from "../lib/links";
import { ipc } from "../lib/ipc";
import { useAppStore } from "../store";
import { Button, IconButton } from "./ui";
import { JobList } from "./JobList";

export function Done() {
  const { t } = useTranslation();
  const { files, jobIds, reports, progress, supportDismissed, targets, target } = useAppStore();
  const { reset, dismissSupport } = useAppStore();
  const jobs = files.filter((f) => jobIds.includes(f.id));
  const count = (state: string) => jobs.filter((f) => progress[f.id]?.state === state).length;
  const year = targets.find((x) => x.major === target)?.name ?? "";

  return (
    <div className="flex h-full flex-col gap-3">
      <div className="flex items-center gap-4 pb-1">
        <motion.span
          initial={{ opacity: 0, scale: 0.5 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ type: "spring", stiffness: 420, damping: 15 }}
          className="grid size-16 shrink-0 place-items-center rounded-full bg-accent text-accent-fg"
          aria-hidden
        >
          <Check className="size-8" strokeWidth={3.25} />
        </motion.span>
        <div className="min-w-0 flex-1">
          <h2 className="text-title leading-none font-black tracking-display" aria-live="polite">
            {t("done.title", { count: count("done") })}
          </h2>
          <p className="mt-1.5 text-base text-muted">
            {t("done.sub")}
            {count("failed") > 0 && ` ${t("done.failed", { count: count("failed") })}`}
            {count("skipped") > 0 && ` ${t("done.skipped", { count: count("skipped") })}`}
          </p>
        </div>
        <Button onClick={reset}>
          <Plus className="size-4" aria-hidden strokeWidth={2.5} />
          {t("done.convertMore")}
        </Button>
      </div>
      <JobList files={jobs} reports={reports} progress={progress} year={year} finished />
      {!supportDismissed && (
        <div className="mt-auto flex items-center gap-3 rounded-full py-2 pr-2 pl-4 text-sm text-muted shadow-outline">
          <Heart className="size-4.5 shrink-0 text-danger" aria-hidden />
          <span className="min-w-0 flex-1">
            {t("done.supportLine")}{" "}
            <Button variant="link" className="min-h-0! text-fg" onClick={() => void ipc.openUrl(REPO_URL)}>
              {t("done.support")}
            </Button>
          </span>
          <IconButton label={t("done.dismiss")} onClick={dismissSupport} className="size-8!">
            <X className="size-4" aria-hidden />
          </IconButton>
        </div>
      )}
    </div>
  );
}
