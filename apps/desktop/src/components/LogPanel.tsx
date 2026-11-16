import * as Switch from "@radix-ui/react-switch";
import { Check, CircleX, Copy, Info, ShieldCheck, TriangleAlert, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { version } from "../../package.json";
import i18n from "../i18n";
import { buildReport, renderMessage, type LogEntry, type LogLevel } from "../lib/activity-log";
import { copyText } from "../lib/clipboard";
import { ipc } from "../lib/ipc";
import { useAppStore } from "../store";
import { Button, IconButton } from "./ui";

export const LOG_TOGGLE_ID = "log-toggle";
export const LOG_PANEL_ID = "activity-log";

const LEVELS: Record<LogLevel, { icon: typeof Info; tone: string; row: string }> = {
  info: { icon: Info, tone: "text-muted", row: "" },
  warning: {
    icon: TriangleAlert,
    tone: "text-warn",
    row: "shadow-[inset_3px_0_0_var(--color-warn)]",
  },
  error: {
    icon: CircleX,
    tone: "text-danger",
    row: "bg-[color-mix(in_srgb,var(--color-danger)_12%,transparent)] shadow-[inset_3px_0_0_var(--color-danger)]",
  },
};

const clock = (time: number) => new Date(time).toLocaleTimeString([], { hour12: false });

function Entry({ entry, focused }: { entry: LogEntry; focused: boolean }) {
  const { t } = useTranslation();
  const look = LEVELS[entry.level];
  const Icon = look.icon;
  return (
    <li
      data-entry={entry.id}
      className={`grid grid-cols-[auto_1rem_minmax(0,1fr)] items-start gap-x-2.5 py-1.5 pr-3 pl-3 ${look.row} ${
        focused ? "outline-2 -outline-offset-2 outline-strong" : ""
      }`}
    >
      <time className="pt-px font-mono text-xs text-muted tabular-nums" dateTime={new Date(entry.time).toISOString()}>
        {clock(entry.time)}
      </time>
      <Icon className={`mt-0.5 size-4 ${look.tone}`} aria-hidden strokeWidth={2.25} />
      <span className="min-w-0 text-sm">
        <span className="sr-only">{t(`log.level.${entry.level}`)}: </span>
        <span className={`break-words ${entry.level === "info" ? "" : "font-bold"}`}>
          {renderMessage(entry, t, "name")}
        </span>
        {entry.detail && (
          <span className="mt-0.5 block font-mono text-xs break-words text-muted select-text">{entry.detail}</span>
        )}
      </span>
    </li>
  );
}

function useReportCopy() {
  const [status, setStatus] = useState<"idle" | "copied" | "failed">("idle");
  const timer = useRef<ReturnType<typeof setTimeout>>(undefined);
  useEffect(() => () => clearTimeout(timer.current), []);

  async function copy() {
    const { log, includePaths, settings } = useAppStore.getState();
    const system = await ipc.systemInfo().catch(() => ({ os: "unknown", arch: "unknown" }));
    const report = buildReport({
      version,
      ...system,
      language: settings.language,
      theme: settings.theme,
      entries: log,
      t: i18n.getFixedT("en"),
      includePaths,
      now: new Date(),
    });
    try {
      await copyText(report);
      setStatus("copied");
    } catch {
      setStatus("failed");
    }
    clearTimeout(timer.current);
    timer.current = setTimeout(() => setStatus("idle"), 2500);
  }
  return { status, copy };
}

function LogBody() {
  const { t } = useTranslation();
  const log = useAppStore((s) => s.log);
  const focusId = useAppStore((s) => s.logFocusId);
  const focusTick = useAppStore((s) => s.logFocusTick);
  const includePaths = useAppStore((s) => s.includePaths);
  const setIncludePaths = useAppStore((s) => s.setIncludePaths);
  const setLogOpen = useAppStore((s) => s.setLogOpen);
  const { status, copy } = useReportCopy();
  const scroller = useRef<HTMLDivElement>(null);
  const stuck = useRef(true);

  // Stay at the newest entry while the reader is there; a "Details" jump goes to its entry instead.
  useLayoutEffect(() => {
    const box = scroller.current;
    if (!box) return;
    const target = focusId === null ? null : box.querySelector<HTMLElement>(`[data-entry="${focusId}"]`);
    if (target) {
      // Not scrollIntoView: it would also scroll the clipped panel itself.
      box.scrollTop = target.offsetTop - (box.clientHeight - target.offsetHeight) / 2;
      stuck.current = false;
    } else if (stuck.current) {
      box.scrollTop = box.scrollHeight;
    }
  }, [log.length, focusId, focusTick]);

  const onScroll = () => {
    const box = scroller.current;
    if (box) stuck.current = box.scrollHeight - box.scrollTop - box.clientHeight < 40;
  };

  return (
    <>
      <div className="flex flex-wrap items-center gap-x-3 gap-y-1.5 px-4 pt-2.5 pb-1">
        <h2 className="flex items-center gap-2 text-base font-black tracking-tight">
          {t("log.title")}
          <span className="rounded-full bg-accent px-2 py-px font-mono text-xs font-bold text-accent-fg">{log.length}</span>
        </h2>
        <div className="ml-auto flex flex-wrap items-center gap-x-3 gap-y-1.5">
          <label className="flex items-center gap-2 text-sm font-bold" htmlFor="log-full-paths">
            <Switch.Root
              id="log-full-paths"
              checked={includePaths}
              onCheckedChange={setIncludePaths}
              className="relative h-6 w-11 shrink-0 rounded-full bg-toggle-off transition-colors data-[state=checked]:bg-strong"
            >
              <Switch.Thumb className="block size-4.5 translate-x-1 rounded-full bg-white transition-transform duration-200 ease-pop data-[state=checked]:translate-x-6 data-[state=checked]:bg-on-strong" />
            </Switch.Root>
            {t("log.fullPaths")}
          </label>
          <Button onClick={() => void copy()} className="min-h-8 px-3.5">
            {status === "copied" ? (
              <Check className="size-4" aria-hidden strokeWidth={3} />
            ) : (
              <Copy className="size-4" aria-hidden />
            )}
            {status === "copied" ? t("log.copied") : status === "failed" ? t("log.copyFailed") : t("log.copy")}
          </Button>
          <IconButton label={t("log.close")} onClick={() => setLogOpen(false)} className="size-8!">
            <X className="size-4" aria-hidden />
          </IconButton>
        </div>
      </div>
      <p
        className={`flex items-center gap-2 px-4 pb-2 text-xs ${includePaths ? "font-bold text-warn" : "text-muted"}`}
      >
        <ShieldCheck className="size-3.5 shrink-0" aria-hidden />
        {includePaths ? t("log.containsFull") : t("log.contains")}
      </p>
      <span role="status" className="sr-only">
        {status === "copied" ? t("log.copied") : status === "failed" ? t("log.copyFailed") : ""}
      </span>
      {/* A scrollable region has to be focusable so the keyboard can scroll it. */}
      <div
        ref={scroller}
        onScroll={onScroll}
        tabIndex={0}
        role="region"
        aria-label={t("log.title")}
        className="relative min-h-0 flex-1 overflow-x-hidden overflow-y-auto overscroll-contain border-t border-line -outline-offset-2"
      >
        {log.length === 0 ? (
          <p className="px-4 py-5 text-sm text-muted">{t("log.empty")}</p>
        ) : (
          <ol className="py-1">
            {log.map((entry) => (
              <Entry key={entry.id} entry={entry} focused={entry.id === focusId} />
            ))}
          </ol>
        )}
      </div>
    </>
  );
}

/** Bottom panel under the main screen. Mounted only while open, so it takes no space when collapsed. */
export function LogPanel() {
  const open = useAppStore((s) => s.logOpen);
  const setLogOpen = useAppStore((s) => s.setLogOpen);
  return (
    <AnimatePresence initial={false}>
      {open && (
        <motion.section
          id={LOG_PANEL_ID}
          aria-label={i18n.t("log.title")}
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 20 }}
          transition={{ duration: 0.22, ease: [0.2, 0.9, 0.2, 1] }}
          onKeyDown={(e) => {
            if (e.key !== "Escape") return;
            setLogOpen(false);
            document.getElementById(LOG_TOGGLE_ID)?.focus();
          }}
          className="mx-6 mb-3 flex h-[clamp(10.5rem,31vh,20rem)] shrink-0 flex-col overflow-hidden rounded-lg bg-surface shadow-outline max-[820px]:mx-4 [@media(max-height:640px)]:absolute [@media(max-height:640px)]:inset-x-0 [@media(max-height:640px)]:bottom-0 [@media(max-height:640px)]:z-20 [@media(max-height:640px)]:h-[clamp(11rem,52vh,22rem)] [@media(max-height:640px)]:shadow-[0_0_0_2px_var(--color-line-strong),0_12px_40px_rgb(0_0_0/0.35)]"
        >
          <LogBody />
        </motion.section>
      )}
    </AnimatePresence>
  );
}

/** Small "Details" button on a failed or blocked row: opens the log at that file's entry. */
export function DetailsButton({ fileId, name, className = "" }: { fileId: string; name: string; className?: string }) {
  const { t } = useTranslation();
  const showLogFor = useAppStore((s) => s.showLogFor);
  return (
    <button
      type="button"
      aria-label={t("log.detailsFor", { file: name })}
      onClick={() => showLogFor(fileId)}
      className={`shrink-0 rounded-sm px-1.5 py-0.5 text-xs font-extrabold text-fg underline decoration-2 underline-offset-4 hover:bg-hover active:bg-press ${className}`}
    >
      {t("log.details")}
    </button>
  );
}
