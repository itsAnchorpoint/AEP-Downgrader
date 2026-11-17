import { animate, motion, useMotionValue, useTransform } from "motion/react";
import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../store";
import { JobList } from "./JobList";
import { PrivacyLine } from "./ui";

/** Overall progress, counted up smoothly each time a file finishes. */
function Percent({ value }: { value: number }) {
  const shown = useMotionValue(0);
  const text = useTransform(shown, (v) => `${Math.round(v)}%`);
  useEffect(() => {
    const controls = animate(shown, value, { duration: 0.5, ease: [0.2, 0.9, 0.2, 1] });
    return () => controls.stop();
  }, [shown, value]);
  return (
    <motion.span aria-hidden className="font-mono text-5xl leading-none font-black tracking-display text-accent-text tabular-nums">
      {text}
    </motion.span>
  );
}

export function Converting() {
  const { t } = useTranslation();
  const { files, jobIds, reports, progress, targets, target } = useAppStore();
  const jobs = files.filter((f) => jobIds.includes(f.id));
  const finished = jobIds.filter((id) => ["done", "failed", "skipped"].includes(progress[id]?.state)).length;
  const year = targets.find((x) => x.major === target)?.name ?? "";

  return (
    <div className="flex h-full flex-col gap-3">
      <div className="flex items-center justify-between gap-3">
        <h2 className="flex items-center gap-3 text-title leading-none font-black tracking-display">
          {t("converting.title")}
          <span
            className="rounded-full bg-raised px-2.5 py-1 text-sm font-extrabold tracking-normal text-fg tabular-nums"
            aria-live="polite"
          >
            {t("converting.progress", { current: Math.min(finished + 1, jobs.length), total: jobs.length })}
          </span>
        </h2>
        <Percent value={jobs.length ? (finished / jobs.length) * 100 : 0} />
      </div>
      <JobList files={jobs} reports={reports} progress={progress} year={year} finished={false} />
      <div className="mt-auto border-t border-line pt-3">
        <PrivacyLine />
      </div>
    </div>
  );
}
