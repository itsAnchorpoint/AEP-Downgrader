import { Check, CircleAlert, Heart, Plus, Repeat, X } from "lucide-react";
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
  const { reset, backToQueue, dismissSupport } = useAppStore();
  const jobs = files.filter((f) => jobIds.includes(f.id));
  const count = (state: string) => jobs.filter((f) => progress[f.id]?.state === state).length;
  const year = targets.find((x) => x.major === target)?.name ?? "";
  const done = count("done");
  const failed = count("failed");
  const skipped = count("skipped");
  const cancelled = count("cancelled");
  // Three endings: everything converted, some failed or stopped, nothing converted. Only the first one celebrates.
  const outcome = done === 0 ? "none" : failed > 0 || cancelled > 0 ? "partial" : "all";

  return (
    <div className="flex h-full flex-col gap-3">
      <div className="flex items-center gap-4 pb-1">
        <motion.span
          initial={{ opacity: 0, scale: 0.6 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={outcome === "all" ? { type: "spring", stiffness: 420, damping: 15 } : { duration: 0.25 }}
          className={`grid size-16 shrink-0 place-items-center rounded-full ${
            outcome === "all" ? "fill-accent" : failed > 0 ? "surface text-warn" : "surface text-muted"
          }`}
          aria-hidden
        >
          {outcome === "all" ? (
            <Check className="size-8" strokeWidth={3.25} />
          ) : (
            <CircleAlert className="size-8" strokeWidth={2.5} />
          )}
        </motion.span>
        <div className="min-w-0 flex-1">
          <h2 className="text-title leading-none font-extrabold tracking-display" aria-live="polite">
            {outcome === "none" ? t("done.none") : t("done.title", { count: done })}
          </h2>
          <p className="mt-1.5 text-base text-muted">
            {outcome !== "none" && t("done.sub")}
            {failed > 0 && <b className="font-bold text-danger"> {t("done.failed", { count: failed })}</b>}
            {skipped > 0 && ` ${t("done.skipped", { count: skipped })}`}
            {cancelled > 0 && ` ${t("done.cancelled", { count: cancelled })}`}
          </p>
        </div>
        <div className="flex shrink-0 gap-2">
          <Button onClick={backToQueue}>
            <Repeat className="size-4" aria-hidden strokeWidth={2.5} />
            {t("done.again")}
          </Button>
          <Button onClick={reset}>
            <Plus className="size-4" aria-hidden strokeWidth={2.5} />
            {t("done.convertMore")}
          </Button>
        </div>
      </div>
      <JobList files={jobs} reports={reports} progress={progress} year={year} finished />
      {!supportDismissed && outcome !== "none" && (
        <div className="mt-auto flex items-center gap-3 rounded-full py-2 pr-2 pl-4 text-sm text-muted surface">
          <Heart className="size-4.5 shrink-0 text-accent-text" aria-hidden />
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
