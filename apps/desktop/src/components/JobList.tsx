import { ArrowRight, Check, CircleAlert, CircleX, Clock, FolderOpen, MinusCircle } from "lucide-react";
import { motion } from "motion/react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import type { FileInfo, Preflight, ProgressEvent } from "../lib/ipc-types";
import { ipc } from "../lib/ipc";
import { Button } from "./ui";
import { FileTypeIcon } from "./FileRow";
import { DetailsButton } from "./LogPanel";

function ProgressTrack({ state }: { state: ProgressEvent["state"] }) {
  const { t } = useTranslation();
  return (
    <div
      role="progressbar"
      aria-label={t(`state.${state}`)}
      aria-valuetext={t(`state.${state}`)}
      className="mb-3 ml-16 h-2.5 overflow-hidden rounded-full bg-raised max-[820px]:ml-0"
    >
      {state === "running" && <div className="indeterminate h-full w-2/5 rounded-full bg-strong" />}
      {state !== "running" && (
        <motion.div
          initial={false}
          animate={{ scaleX: state === "done" ? 1 : 0 }}
          transition={{ duration: 0.45, ease: [0.2, 0.9, 0.2, 1] }}
          style={{ originX: 0 }}
          className="h-full rounded-full bg-ok"
        />
      )}
    </div>
  );
}

function Status({ state }: { state: ProgressEvent["state"] }) {
  const { t } = useTranslation();
  const look: Record<ProgressEvent["state"], { icon: ReactNode; tone: string }> = {
    queued: { icon: <Clock className="size-4.5" />, tone: "text-muted" },
    running: { icon: null, tone: "text-fg" },
    done: { icon: <Check className="size-4.5" strokeWidth={3} />, tone: "text-ok" },
    failed: { icon: <CircleX className="size-4.5" />, tone: "text-danger" },
    skipped: { icon: <MinusCircle className="size-4.5" />, tone: "text-muted" },
  };
  return (
    <span className={`flex items-center gap-1.5 text-sm font-extrabold ${look[state].tone}`}>
      <span aria-hidden>{look[state].icon}</span>
      {t(`state.${state}`)}
    </span>
  );
}

/** One line under a finished file: why it needs attention. Clean results say nothing. */
function Outcome({ event, warned, file }: { event: ProgressEvent; warned: boolean; file: FileInfo }) {
  const { t } = useTranslation();
  const details = <DetailsButton fileId={file.id} name={file.name} />;
  if (event.state === "failed") {
    return (
      <p className="flex items-center gap-2 pb-3 pl-16 text-sm font-bold text-danger max-[820px]:pl-0">
        <CircleX className="size-4.5 shrink-0" aria-hidden />
        {t(`error.${event.errorCode ?? "unknown"}`, { defaultValue: t("error.unknown") })}
        {details}
      </p>
    );
  }
  if (event.state === "skipped") {
    return (
      <p className="flex items-center gap-2 pb-3 pl-16 text-sm font-bold text-muted max-[820px]:pl-0">
        <MinusCircle className="size-4.5 shrink-0" aria-hidden />
        {t("state.skipped")}
        {details}
      </p>
    );
  }
  if (event.state === "done" && warned) {
    return (
      <p className="flex items-center gap-2 pb-3 pl-16 text-sm font-bold text-warn max-[820px]:pl-0">
        <CircleAlert className="size-4.5 shrink-0" aria-hidden />
        {t("done.warning")}
        {details}
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
    <ul className="min-h-0 overflow-x-hidden overflow-y-auto overscroll-contain border-t-2 border-line-strong">
      {files.map((file, index) => {
        const event = progress[file.id] ?? { id: file.id, state: "queued" as const };
        const warned = reports[file.id]?.notes.some((n) => n.severity === "warn") ?? false;
        const name = finished && event.outputPath ? outputFileName(event.outputPath) : file.name;
        return (
          <motion.li
            key={file.id}
            initial={{ opacity: 0, x: -28 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, ease: [0.2, 0.9, 0.2, 1], delay: Math.min(index, 6) * 0.06 }}
            className={`border-b border-line ${!finished && event.state === "queued" ? "text-muted" : ""}`}
          >
            <div className="grid min-h-[62px] grid-cols-[3rem_minmax(0,1fr)_auto] items-center gap-x-4 py-2 pr-1 pl-1">
              <FileTypeIcon file={file} />
              <span className="min-w-0">
                <span className="block truncate text-lg font-extrabold tracking-tight" title={name}>
                  {name}
                </span>
                <span className="flex items-center gap-1.5 font-mono text-sm text-muted">
                  {file.version?.name}
                  <ArrowRight className="size-3.5" aria-hidden />
                  {year}
                </span>
              </span>
              {finished ? (
                event.state === "done" &&
                event.outputPath && (
                  <Button onClick={() => void ipc.revealInFolder(event.outputPath!)}>
                    <FolderOpen className="size-4" aria-hidden />
                    {t("done.showFolder")}
                  </Button>
                )
              ) : (
                <Status state={event.state} />
              )}
            </div>
            {finished ? <Outcome event={event} warned={warned} file={file} /> : <ProgressTrack state={event.state} />}
          </motion.li>
        );
      })}
    </ul>
  );
}
