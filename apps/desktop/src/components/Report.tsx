import { Box, CircleAlert, CircleCheck, Info, MinusCircle, OctagonX, Waves, ShieldCheck } from "lucide-react";
import { motion } from "motion/react";
import { useTranslation } from "react-i18next";
import type { Note, Preflight } from "../lib/ipc-types";
import { useAppStore } from "../store";
import { StatusBadge } from "./ui";

type Look = { icon: typeof Info; color: string };

const BY_SEVERITY: Record<Note["severity"], Look> = {
  info: { icon: Info, color: "text-muted" },
  warn: { icon: CircleAlert, color: "text-warn" },
  block: { icon: OctagonX, color: "text-danger" },
};
/** Notes about things that disappear or are kept get their own icon; the rest follow severity. */
const BY_CODE: Record<string, Look> = {
  "shadow-color-removed": { icon: MinusCircle, color: "text-danger" },
  "effects-removed": { icon: Waves, color: "text-danger" },
  "3d-material-removed": { icon: MinusCircle, color: "text-danger" },
  "newer-data-removed": { icon: MinusCircle, color: "text-danger" },
  "plugins-kept": { icon: Box, color: "text-warn" },
  "no-changes": { icon: CircleCheck, color: "text-ok" },
};

/** Notes carry a code and params; the text always comes from the catalog. */
function useNoteText() {
  const { t, i18n } = useTranslation();
  return (note: Note) => {
    const params = Object.fromEntries(
      Object.entries(note.params).map(([k, v]) => [k, Array.isArray(v) ? v.join(", ") : v]),
    );
    const hintKey = `note.${note.code}-hint`;
    return {
      text: t(`note.${note.code}`, { defaultValue: t("note.unknown", { code: note.code }), ...params }),
      hint: i18n.exists(hintKey) ? t(hintKey) : null,
    };
  };
}

export function Report({ id, report, year }: { id: string; report?: Preflight; year: string }) {
  const { t } = useTranslation();
  const noteText = useNoteText();
  // A successful conversion without notes means nothing will be lost.
  const notes: Note[] = report?.notes.length
    ? report.notes
    : [{ severity: "info", code: "no-changes", params: { year } }];
  const status = useAppStore((s) => s.targets.find((x) => x.major === report?.target)?.status);

  return (
    <motion.div
      id={id}
      initial={{ opacity: 0, y: -6 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.24, ease: [0.2, 0.8, 0.2, 1] }}
      className="mb-2 ml-16 border-l-2 border-line-strong py-1 pl-4 text-sm max-[820px]:ml-6"
    >
      <h3 className="mb-2 flex flex-wrap items-center gap-x-2.5 gap-y-1 text-base font-extrabold">
        {report ? (
          <>
            {t("report.for")}
            <span className="font-mono">{t("target.ae", { name: year })}</span>
            {status && <StatusBadge status={status} pill />}
          </>
        ) : (
          t("report.loading")
        )}
      </h3>
      {report && (
        <ul className="mb-2 flex flex-col gap-1.5">
          {notes.map((note) => {
            const { icon: Icon, color } = BY_CODE[note.code] ?? BY_SEVERITY[note.severity];
            const { text, hint } = noteText(note);
            return (
              <li key={note.code} className="flex items-start gap-2.5">
                <Icon className={`mt-px size-5 shrink-0 ${color}`} aria-hidden strokeWidth={2.25} />
                <span>
                  <span className={note.severity === "info" ? "" : "font-bold"}>{text}</span>
                  {hint && <span className="block text-xs text-muted">{hint}</span>}
                </span>
              </li>
            );
          })}
        </ul>
      )}
      <p className="flex items-center gap-2 text-xs text-muted">
        <ShieldCheck className="size-3.5 shrink-0" aria-hidden />
        {t("report.safe")}
      </p>
    </motion.div>
  );
}
