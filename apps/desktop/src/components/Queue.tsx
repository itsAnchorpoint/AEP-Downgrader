import { ArrowRight, Plus, ShieldCheck } from "lucide-react";
import { AnimatePresence } from "motion/react";
import { useTranslation } from "react-i18next";
import { isConvertible, useAppStore } from "../store";
import { FileRow } from "./FileRow";
import { Button, StatusBadge } from "./ui";

function TargetPicker() {
  const { t } = useTranslation();
  const targets = useAppStore((s) => s.targets);
  const target = useAppStore((s) => s.target);
  const setTarget = useAppStore((s) => s.setTarget);

  return (
    <div role="group" aria-labelledby="target-label">
      <p id="target-label" className="mb-2 text-sm font-extrabold tracking-[0.1em] text-muted uppercase">
        {t("queue.convertTo")} <b className="font-extrabold text-fg">{t("file.versionPlain")}</b>
      </p>
      <div className="flex gap-2">
        {targets.map((x) => {
          const on = x.major === target;
          return (
            <button
              key={x.major}
              type="button"
              aria-pressed={on}
              onClick={() => void setTarget(x.major)}
              className={`flex min-w-0 flex-1 flex-col items-start rounded-md px-3 py-1.5 transition-[translate,scale,background-color] duration-250 ease-pop max-[820px]:px-2 ${
                on
                  ? "-translate-y-[3px] bg-strong text-on-strong"
                  : "shadow-outline hover:-translate-y-[3px] hover:bg-hover active:scale-95"
              }`}
            >
              <span className="font-mono text-2xl leading-tight font-black tracking-tight max-[820px]:text-xl">
                {x.name}
              </span>
              <span className={on ? "[&_span]:text-inherit" : ""}>
                <StatusBadge status={x.status} />
              </span>
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

  return (
    <div className="flex h-full flex-col gap-3">
      <div className="flex items-center justify-between gap-3">
        <h2 className="flex items-center gap-3 text-title leading-none font-black tracking-display">
          {t("queue.files")}
          <span className="rounded-full bg-accent px-2.5 py-1 text-sm font-extrabold tracking-normal text-accent-fg">
            {files.length}
          </span>
        </h2>
        <div className="flex gap-2">
          <Button onClick={() => void browse()}>
            <Plus className="size-4" aria-hidden strokeWidth={2.5} />
            {t("queue.addMore")}
          </Button>
          <Button onClick={clearFiles}>{t("queue.clear")}</Button>
        </div>
      </div>

      <ul className="min-h-0 flex-1 overflow-x-hidden overflow-y-auto overscroll-contain border-t-2 border-line-strong">
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

      <div className="flex flex-col gap-2.5 border-t-2 border-line pt-3">
        <TargetPicker />
        <div className="flex items-center gap-4">
          <p className="flex min-w-0 flex-1 items-center gap-2 text-sm text-muted max-[820px]:hidden">
            <ShieldCheck className="size-4 shrink-0" aria-hidden />
            <span className="truncate">
              {settings.location === "folder" && settings.folder
                ? t("queue.saveFolder", { folder: settings.folder })
                : t("queue.saveNext")}
            </span>
          </p>
          <Button
            variant="primary"
            disabled={count === 0}
            onClick={() => void convert()}
            className="group/convert h-14 px-8! text-xl tracking-tight max-[820px]:flex-1"
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
