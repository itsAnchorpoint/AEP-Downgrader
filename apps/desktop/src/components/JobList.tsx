import { ArrowRight, Ban, Check, CircleAlert, CircleX, Clock, FolderOpen, LoaderCircle, MinusCircle } from "lucide-react";
import { motion } from "motion/react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { aeMajorForFormat } from "../lib/format";
import { platform } from "../lib/platform";
import type { FileInfo, Note, Preflight, ProgressEvent } from "../lib/ipc-types";
import { ipc } from "../lib/ipc";
import { Button } from "./ui";
import { FileTypeIcon } from "./FileRow";
import { DetailsButton } from "./LogPanel";
import { useNoteText } from "./Report";

/** Finder on macOS, Explorer on Windows, a folder elsewhere. */
const SHOW_IN = platform === "mac" ? "done.showFinder" : platform === "windows" ? "done.showExplorer" : "done.showFolder";

/** The version a file was saved in, as a year; presets only know their format. */
function sourceYear(file: FileInfo) {
  if (file.version?.name) return file.version.name;
  const major = file.format ? aeMajorForFormat(file.format.major) : null;
  return major !== null ? String(2000 + major) : "";
}

function ProgressTrack({ state }: { state: ProgressEvent["state"] }) {
  const { t } = useTranslation();
  return (
    <div
      role="progressbar"
      aria-label={t(`state.${state}`)}
      aria-valuetext={t(`state.${state}`)}
      className="mb-3 ml-[68px] h-2 overflow-hidden rounded-full bg-raised max-[820px]:ml-0"
    >
      {state === "running" && <div className="indeterminate h-full w-2/5 rounded-full bg-strong" />}
      {state !== "running" && (
        <motion.div
          initial={false}
          animate={{ scaleX: state === "done" ? 1 : 0 }}
          transition={{ duration: 0.45, ease: [0.2, 0.9, 0.2, 1] }}
          style={{ originX: 0 }}
          className="h-full rounded-full bg-fg/30"
        />
      )}
    </div>
  );
}

function Status({ state }: { state: ProgressEvent["state"] }) {
  const { t } = useTranslation();
  const look: Record<ProgressEvent["state"], { icon: ReactNode; tone: string }> = {
    queued: { icon: <Clock className="size-4.5" />, tone: "text-muted" },
    running: { icon: <LoaderCircle className="size-4.5 animate-spin" />, tone: "text-fg" },
    done: { icon: <Check className="size-4.5" strokeWidth={3} />, tone: "text-ok" },
    failed: { icon: <CircleX className="size-4.5" />, tone: "text-danger" },
    skipped: { icon: <MinusCircle className="size-4.5" />, tone: "text-muted" },
    cancelled: { icon: <Ban className="size-4.5" />, tone: "text-muted" },
  };
  return (
    <span className={`flex items-center gap-1.5 text-sm font-extrabold ${look[state].tone}`}>
      <span aria-hidden>{look[state].icon}</span>
      {t(`state.${state}`)}
    </span>
  );
}

/** One line under a finished file: why it needs attention. Clean results say nothing. */
function Outcome({ event, warnings, file }: { event: ProgressEvent; warnings: Note[]; file: FileInfo }) {
  const { t } = useTranslation();
  const noteText = useNoteText();
  const details = <DetailsButton fileId={file.id} name={file.name} />;
  const line = "flex items-start gap-2 pb-3 pl-[68px] text-sm max-[820px]:pl-0";
  if (event.state === "failed") {
    return (
      <p className={`${line} font-bold text-danger`}>
        <CircleX className="mt-px size-4.5 shrink-0" aria-hidden />
        <span>
          {t(`error.${event.errorCode ?? "unknown"}`, { defaultValue: t("error.unknown") })} {details}
        </span>
      </p>
    );
  }
  if (event.state === "skipped") {
    return (
      <p className={`${line} font-bold text-muted`}>
        <MinusCircle className="mt-px size-4.5 shrink-0" aria-hidden />
        {t("done.skippedExists")}
      </p>
    );
  }
  if (event.state === "cancelled") {
    return (
      <p className={`${line} font-bold text-muted`}>
        <Ban className="mt-px size-4.5 shrink-0" aria-hidden />
        {t("done.cancelledFile")}
      </p>
    );
  }
  if (event.state === "done" && warnings.length > 0) {
    // The notes describe the conversion ahead of time; here they become a count, the list on hover.
    return (
      <p className={`${line} font-bold text-warn`} title={warnings.map((n) => noteText(n).text).join("\n")}>
        <CircleAlert className="mt-px size-4.5 shrink-0" aria-hidden />
        <span className="underline decoration-dotted decoration-1 underline-offset-4">
          {t("done.changes", { count: warnings.length })}
        </span>
      </p>
    );
  }
  return null;
}

const outputFileName = (path: string) => path.split(/[\\/]/).pop() ?? path;

export function JobList({
  files,
  reports,
  progress,
  year,
  finished,
}: {
  files: FileInfo[];
  reports: Record<string, Preflight>;
  progress: Record<string, ProgressEvent>;
  year: string;
  /** Finished lists show the result and a folder button; running lists show progress bars. */
  finished: boolean;
}) {
  const { t } = useTranslation();
  return (
    <ul className="min-h-0 overflow-x-hidden overflow-y-auto overscroll-contain scroll-fade pb-4 border-t border-line">
      {files.map((file, index) => {
        const event = progress[file.id] ?? { id: file.id, state: "queued" as const };
        // What this file lost; the experimental note is about the version, not the file.
        const warnings = reports[file.id]?.notes.filter((n) => n.severity === "warn" && n.code !== "experimental-target") ?? [];
        const name = finished && event.outputPath ? outputFileName(event.outputPath) : file.name;
        return (
          <motion.li
            key={file.id}
            initial={finished ? false : { opacity: 0, x: -16 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.3, ease: [0.22, 1, 0.36, 1], delay: Math.min(index, 6) * 0.04 }}
            className={`border-b border-line ${event.state === "queued" || event.state === "cancelled" ? "text-muted" : ""}`}
          >
            <div className="grid min-h-[62px] grid-cols-[3rem_minmax(0,1fr)_auto] items-center gap-x-4 py-2 pr-1 pl-1">
              <FileTypeIcon file={file} />
              <span className="min-w-0">
                <span className="block truncate text-[17px] font-extrabold tracking-tight select-text" title={name}>
                  {name}
                </span>
                <span className="flex items-center gap-1.5 font-mono text-sm text-muted">
                  {sourceYear(file)}
                  <ArrowRight className="size-3.5" aria-hidden />
                  {year}
                </span>
              </span>
              {finished ? (
                event.state === "done" &&
                event.outputPath && (
                  <Button onClick={() => void ipc.revealInFolder(event.outputPath!)}>
                    <FolderOpen className="size-4" aria-hidden />
                    {t(SHOW_IN)}
                  </Button>
                )
              ) : (
                <Status state={event.state} />
              )}
            </div>
            {finished ? <Outcome event={event} warnings={warnings} file={file} /> : <ProgressTrack state={event.state} />}
          </motion.li>
        );
      })}
    </ul>
  );
}
