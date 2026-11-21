import { ArrowRight, FlaskConical, Info, Plus, ShieldCheck } from "lucide-react";
import type { KeyboardEvent } from "react";
import { AnimatePresence } from "motion/react";
import { useTranslation } from "react-i18next";
import { shortcut } from "../lib/platform";
import { isConvertible, useAppStore } from "../store";
import { FileRow } from "./FileRow";
import { Button, StatusBadge } from "./ui";

function TargetPicker() {
  const { t } = useTranslation();
  const targets = useAppStore((s) => s.targets);
  const target = useAppStore((s) => s.target);
  const setTarget = useAppStore((s) => s.setTarget);
  const current = targets.findIndex((x) => x.major === target);

  // One choice out of a few: a radio group, arrow keys move the choice.
  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    const step = e.key === "ArrowRight" || e.key === "ArrowDown" ? 1 : e.key === "ArrowLeft" || e.key === "ArrowUp" ? -1 : 0;
    if (!step || current < 0) return;
    e.preventDefault();
    const next = targets[(current + step + targets.length) % targets.length];
    void setTarget(next.major);
    (e.currentTarget.querySelector(`[data-major="${next.major}"]`) as HTMLElement | null)?.focus();
  };

  return (
    <div>
      <p id="target-label" className="mb-2 text-sm font-bold text-muted">
        {t("queue.convertTo")} <b className="font-extrabold text-fg">{t("file.versionPlain")}</b>
      </p>
      <div role="radiogroup" aria-labelledby="target-label" className="flex gap-2" onKeyDown={onKeyDown}>
        {targets.map((x) => {
          const on = x.major === target;
          return (
            <button
              key={x.major}
              type="button"
              role="radio"
              aria-checked={on}
              tabIndex={on ? 0 : -1}
              data-major={x.major}
              title={t(x.status === "verified" ? "target.verifiedHint" : "target.experimentalHint")}
              onClick={() => void setTarget(x.major)}
              className={`flex min-w-0 flex-1 flex-col items-start rounded-md px-3 py-1.5 transition-[background-color,box-shadow,scale] duration-200 active:scale-[0.97] max-[820px]:px-2 ${
                on ? "bg-raised shadow-[inset_0_0_0_2px_var(--color-fg)]" : "surface hover:bg-hover"
              }`}
            >
              <span className="font-mono text-2xl leading-tight font-extrabold max-[820px]:text-xl">{x.name}</span>
              <StatusBadge status={x.status} quiet={!on} />
            </button>
          );
        })}
      </div>
    </div>
  );
}

export function Queue() {
  const { t } = useTranslation();
  const { files, reports, expandedId, target, targets, settings } = useAppStore();
  const { browse, clearFiles, removeFile, toggleExpanded, convert } = useAppStore();
  const year = targets.find((x) => x.major === target)?.name ?? "";
  const count = files.filter((f) => isConvertible(f, reports)).length;
  const experimental = targets.find((x) => x.major === target)?.status === "experimental";

  return (
    <div className="flex h-full flex-col gap-3">
      <div className="flex items-center justify-between gap-3">
        <h2 className="flex items-center gap-3 text-title leading-none font-extrabold tracking-display">
          {t("queue.files")}
          <span className="rounded-full bg-raised px-2.5 py-1 text-sm font-extrabold tracking-normal text-fg tabular-nums">
            {files.length}
          </span>
        </h2>
        <div className="flex gap-2">
          <Button onClick={() => void browse()} title={`${t("queue.addMore")} (${shortcut("O")})`}>
            <Plus className="size-4" aria-hidden strokeWidth={2.5} />
            {t("queue.addMore")}
          </Button>
          <Button variant="ghost" className="px-3" onClick={clearFiles}>
            {t("queue.clear")}
          </Button>
        </div>
      </div>

      <ul className="min-h-0 flex-1 overflow-x-hidden overflow-y-auto overscroll-contain scroll-fade pb-4 border-t border-line">
        <AnimatePresence>
          {files.map((file, index) => (
            <FileRow
              key={file.id}
              file={file}
              report={reports[file.id]}
              year={year}
              index={index}
              expanded={expandedId === file.id}
              onToggle={() => toggleExpanded(file.id)}
              onRemove={() => removeFile(file.id)}
            />
          ))}
        </AnimatePresence>
      </ul>

      <div className="flex flex-col gap-2.5 border-t border-line pt-3">
        <TargetPicker />
        {experimental && count > 0 && (
          <p className="flex items-start gap-2 text-sm font-bold text-warn">
            <FlaskConical className="mt-0.5 size-4 shrink-0" aria-hidden />
            {t("queue.experimental", { year })}
          </p>
        )}
        <div className="flex items-center gap-4">
          <p className="flex min-w-0 flex-1 items-center gap-2 text-sm text-muted">
            {count > 0 ? <ShieldCheck className="size-4 shrink-0" aria-hidden /> : <Info className="size-4 shrink-0" aria-hidden />}
            <span className="truncate">
              {count === 0
                ? t("queue.nothingToConvert", { year })
                : settings.location === "folder" && settings.folder
                  ? t("queue.saveFolder", { folder: settings.folder })
                  : t("queue.saveNext")}
            </span>
          </p>
          <Button
            variant="primary"
            disabled={count === 0}
            title={shortcut("Enter")}
            onClick={() => void convert()}
            className="group/convert h-14 shrink-0 px-8! text-xl tracking-tight max-[820px]:px-6!"
          >
            {count > 0 ? t("queue.convert", { count }) : t("queue.convert")}
            <ArrowRight
              className="size-6 transition-transform duration-250 ease-pop group-hover/convert:translate-x-1.5"
              strokeWidth={3}
              aria-hidden
            />
          </Button>
        </div>
      </div>
    </div>
  );
}
