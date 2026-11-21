import { ChevronDown, CircleCheck, CircleMinus, LoaderCircle, OctagonX, TriangleAlert, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { aeMajorForFormat, formatBytes, versionNumber } from "../lib/format";
import { isIgnored } from "../lib/files";
import type { FileInfo, Preflight } from "../lib/ipc-types";
import { Report } from "./Report";

/** Square tile with the file extension in mono, as on the drop screen; "!" for files that can't be converted. */
export function FileTypeIcon({ file }: { file: FileInfo }) {
  const ext = file.name.split(".").pop()?.slice(0, 4).toLowerCase() ?? "";
  const broken = file.problem !== undefined && !isIgnored(file);
  return (
    <span
      className={`grid size-12 shrink-0 place-items-center rounded-md surface font-mono font-bold ${
        broken ? "text-lg text-danger" : file.problem ? "text-sm text-muted" : "text-sm text-fg"
      }`}
      aria-hidden
    >
      {broken ? "!" : `.${ext}`}
    </span>
  );
}

/** Separates the parts of a meta line with a slash, like the version trail in a file path. */
function Sep({ children }: { children: ReactNode }) {
  return (
    <>
      <span aria-hidden className="text-muted/70">
        /
      </span>
      {children}
    </>
  );
}

/** The version the file was saved in, and its size. Build and system are in the report. */
function MetaLine({ file }: { file: FileInfo }) {
  const { t } = useTranslation();
  const v = file.version;
  if (file.problem) {
    return (
      <span className={`truncate ${isIgnored(file) ? "" : "font-bold text-danger"}`}>{t(`problem.${file.problem}`)}</span>
    );
  }
  let version: ReactNode = null;
  if (v) {
    version = (
      <>
        <span className="truncate text-fg">{v.name ? t("file.version", { name: v.name }) : t("file.versionPlain")}</span>
        <Sep>
          <span className="shrink-0 font-mono text-xs font-bold text-fg">
            {versionNumber(v)}
            {!v.release && ` ${t("file.beta")}`}
          </span>
        </Sep>
      </>
    );
  } else if (file.format) {
    // Presets carry a format number but no version word; the format tells the release.
    const major = aeMajorForFormat(file.format.major);
    version = major !== null && <span className="truncate text-fg">{t("file.version", { name: String(2000 + major) })}</span>;
  }
  return (
    <>
      {version}
      <Sep>
        <span className="shrink-0 font-mono text-xs">{formatBytes(file.size)}</span>
      </Sep>
    </>
  );
}

type Summary = { text: string; tone: string; icon: ReactNode };

/** What the report says, in a word, so nobody has to open it to know whether something is lost. */
function useSummary(file: FileInfo, report?: Preflight): Summary {
  const { t } = useTranslation();
  const icon = "size-4 shrink-0";
  if (file.problem) {
    return isIgnored(file)
      ? { text: t("state.ignored"), tone: "text-muted", icon: <CircleMinus className={icon} aria-hidden /> }
      : { text: t("state.blocked"), tone: "text-danger", icon: <OctagonX className={icon} aria-hidden /> };
  }
  if (!report) {
    return {
      text: t("queue.summary.checking"),
      tone: "text-muted",
      icon: <LoaderCircle className={`${icon} animate-spin`} aria-hidden />,
    };
  }
  if (report.blocked) {
    const nothingToDo = report.notes.some((n) => n.code === "not-a-downgrade" || n.code === "target-not-older");
    return nothingToDo
      ? { text: t("state.nothingToDo"), tone: "text-muted", icon: <CircleMinus className={icon} aria-hidden /> }
      : { text: t("state.blocked"), tone: "text-danger", icon: <OctagonX className={icon} aria-hidden /> };
  }
  // The experimental-target note is about the version, not this file: the target tile says it.
  const changes = report.notes.filter((n) => n.severity !== "info" && n.code !== "experimental-target").length;
  return changes > 0
    ? {
        text: t("queue.summary.changes", { count: changes }),
        tone: "text-warn",
        icon: <TriangleAlert className={icon} aria-hidden />,
      }
    : { text: t("queue.summary.clean"), tone: "text-muted", icon: <CircleCheck className={`${icon} text-ok`} aria-hidden /> };
}

export function FileRow({
  file,
  report,
  year,
  index,
  expanded,
  onToggle,
  onRemove,
}: {
  file: FileInfo;
  report?: Preflight;
  year: string;
  index: number;
  expanded: boolean;
  onToggle: () => void;
  onRemove: () => void;
}) {
  const { t } = useTranslation();
  const reportId = `report-${file.id}`;
  const summary = useSummary(file, report);

  return (
    <motion.li
      initial={{ opacity: 0, x: -16 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.3, ease: [0.22, 1, 0.36, 1], delay: Math.min(index, 6) * 0.04 }}
      className="border-b border-line"
    >
      <div className="grid grid-cols-[minmax(0,1fr)_2.25rem] items-center">
        <button
          type="button"
          aria-expanded={expanded}
          aria-controls={reportId}
          aria-label={`${file.name}: ${summary.text}. ${expanded ? t("queue.hideReport") : t("queue.showReport")}`}
          onClick={onToggle}
          className={`grid min-h-[60px] w-full grid-cols-[3rem_minmax(0,1fr)_auto_1.25rem] items-center gap-x-4 rounded-md py-2 pr-2 pl-1 text-left -outline-offset-2 transition-colors hover:bg-hover active:bg-press ${
            expanded ? "bg-hover" : ""
          }`}
        >
          <FileTypeIcon file={file} />
          <span className="min-w-0">
            <span
              className={`block truncate text-[17px] font-extrabold tracking-tight ${file.problem ? "text-muted" : ""}`}
              title={file.name}
            >
              {file.name}
            </span>
            <span className="flex min-w-0 items-center gap-2 text-sm whitespace-nowrap text-muted">
              <MetaLine file={file} />
            </span>
          </span>
          <span className={`flex items-center gap-1.5 text-sm font-bold whitespace-nowrap ${summary.tone}`}>
            {summary.icon}
            <span className="max-[820px]:sr-only">{summary.text}</span>
          </span>
          <ChevronDown
            className={`size-4.5 text-muted transition-transform duration-200 ease-out-quint ${expanded ? "rotate-180" : ""}`}
            aria-hidden
          />
        </button>
        <button
          type="button"
          aria-label={t("queue.remove")}
          title={t("queue.remove")}
          onClick={onRemove}
          className="grid size-8 place-items-center justify-self-end rounded-sm text-muted/70 transition-colors hover:bg-hover hover:text-fg active:bg-press"
        >
          <X className="size-4" aria-hidden />
        </button>
      </div>
      <AnimatePresence initial={false}>
        {expanded && <Report id={reportId} file={file} report={report} year={year} />}
      </AnimatePresence>
    </motion.li>
  );
}
