import { ChevronDown, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { formatBytes, versionNumber } from "../lib/format";
import type { FileInfo, Preflight } from "../lib/ipc-types";
import { DetailsButton } from "./LogPanel";
import { Report } from "./Report";

/** Square tile with the file extension; presets are amber, problems are outlined in red. */
export function FileTypeIcon({ file }: { file: FileInfo }) {
  const ext = file.name.split(".").pop()?.slice(0, 4).toUpperCase() ?? "";
  const tone = file.problem
    ? "text-danger shadow-[inset_0_0_0_2px_var(--color-danger)] text-xl"
    : file.kind === "preset"
      ? "bg-warn-fill text-ink-black text-xs"
      : "bg-accent text-accent-fg text-xs";
  return (
    <span
      className={`grid size-12 shrink-0 place-items-center rounded-md font-black tracking-wider ${tone}`}
      aria-hidden
    >
      {file.problem ? "!" : ext}
    </span>
  );
}

/** Separates the parts of a meta line with a slash, like the version trail in a file path. */
function Sep({ children }: { children: ReactNode }) {
  return (
    <>
      <span aria-hidden className="text-muted">
        /
      </span>
      {children}
    </>
  );
}

function VersionLine({ file }: { file: FileInfo }) {
  const { t } = useTranslation();
  const v = file.version;
  if (file.problem) return <span className="font-bold text-danger">{t(`problem.${file.problem}`)}</span>;
  if (!v) {
    // Presets carry only a format number until more is known.
    return file.format ? (
      <span className="font-mono text-xs">{t("file.format", { major: file.format.major, minor: file.format.minor })}</span>
    ) : null;
  }
  return (
    <>
      <span className="text-fg">{v.name ? t("file.version", { name: v.name }) : t("file.versionPlain")}</span>
      <Sep>
        <span className="font-mono font-bold text-fg">
          {versionNumber(v)}
          {!v.release && ` ${t("file.beta")}`}
        </span>
      </Sep>
      <Sep>
        <span className="font-mono text-xs">{t("file.build", { build: v.build })}</span>
      </Sep>
      {v.platform !== "unknown" && (
        <Sep>
          <span>{t(`platform.${v.platform}`)}</span>
        </Sep>
      )}
    </>
  );
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
  const nothingToDo = report?.notes.some((n) => n.code === "not-a-downgrade" || n.code === "target-not-older");
  const body = (
    <>
      <FileTypeIcon file={file} />
      <span className="min-w-0">
        <span
          className={`block truncate text-lg font-extrabold tracking-tight ${file.problem ? "text-muted line-through decoration-2" : ""}`}
          title={file.name}
        >
          {file.name}
        </span>
        <span className="flex items-center gap-2 overflow-hidden text-sm whitespace-nowrap text-muted">
          <VersionLine file={file} />
          {report?.blocked &&
            (nothingToDo ? (
              <span className="font-bold text-muted">{t("state.nothingToDo")}</span>
            ) : (
              <span className="font-bold text-danger">{t("state.skipped")}</span>
            ))}
        </span>
      </span>
      {file.problem ? (
        <span className="rounded-full px-2.5 py-0.5 text-xs font-extrabold text-danger shadow-[inset_0_0_0_2px_var(--color-danger)]">
          {t("state.skipped")}
        </span>
      ) : (
        <span className="font-mono text-xs text-muted">{formatBytes(file.size)}</span>
      )}
      {!file.problem && (
        <ChevronDown
          className={`size-4.5 text-muted transition-transform duration-200 ${expanded ? "rotate-180" : ""}`}
          aria-hidden
        />
      )}
    </>
  );
  // Failed or blocked rows get a "Details" button between the row and the remove button.
  const showDetails = Boolean(file.problem || report?.blocked);
  const layout = `grid min-h-[62px] w-full grid-cols-[3rem_minmax(0,1fr)_auto_1.25rem] items-center gap-x-4 py-2 pl-1 ${
    showDetails ? "pr-28" : "pr-11"
  }`;

  return (
    <motion.li
      initial={{ opacity: 0, x: -28 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.5, ease: [0.2, 0.9, 0.2, 1], delay: Math.min(index, 6) * 0.06 }}
      className="border-b border-line"
    >
      <div className="group relative">
        {file.problem ? (
          <div className={`${layout} ${showDetails ? "" : "pr-3"}`}>{body}</div>
        ) : (
          <button
            type="button"
            aria-expanded={expanded}
            aria-controls={reportId}
            onClick={onToggle}
            className={`${layout} text-left -outline-offset-4 transition-colors hover:bg-hover active:bg-press ${
              expanded ? "pl-4 shadow-[inset_4px_0_0_var(--color-strong)]" : ""
            }`}
          >
            {body}
          </button>
        )}
        {showDetails && (
          <DetailsButton fileId={file.id} name={file.name} className="absolute inset-y-0 right-11 my-auto h-fit" />
        )}
        <button
          type="button"
          aria-label={t("queue.remove")}
          title={t("queue.remove")}
          onClick={onRemove}
          className="absolute inset-y-0 right-2 my-auto grid size-7 place-items-center rounded-sm text-muted opacity-0 transition-opacity group-hover:opacity-100 hover:bg-hover hover:text-fg focus-visible:opacity-100"
        >
          <X className="size-3.5" aria-hidden />
        </button>
      </div>
      <AnimatePresence initial={false}>
        {expanded && <Report id={reportId} report={report} year={year} />}
      </AnimatePresence>
    </motion.li>
  );
}
