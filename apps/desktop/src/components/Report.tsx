import { CircleAlert, CircleCheck, CircleMinus, FlaskConical, Info, MinusCircle, OctagonX, Plug, Puzzle, ShieldCheck } from "lucide-react";
import { motion } from "motion/react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { isIgnored } from "../lib/files";
import { versionNumber } from "../lib/format";
import type { FileInfo, Note, Preflight } from "../lib/ipc-types";
import { useAppStore } from "../store";
import { DetailsButton } from "./LogPanel";
import { StatusBadge } from "./ui";

type Look = { icon: typeof Info; color: string };

const BY_SEVERITY: Record<Note["severity"], Look> = {
  info: { icon: Info, color: "text-muted" },
  warn: { icon: CircleAlert, color: "text-warn" },
  block: { icon: OctagonX, color: "text-danger" },
};
/** Notes about things that disappear or are kept get their own icon; the rest follow severity. Red is only for "can't convert". */
const BY_CODE: Record<string, Look> = {
  "shadow-color-removed": { icon: MinusCircle, color: "text-warn" },
  "effects-removed": { icon: Puzzle, color: "text-warn" },
  "3d-material-removed": { icon: MinusCircle, color: "text-warn" },
  "newer-data-removed": { icon: MinusCircle, color: "text-warn" },
  "plugins-kept": { icon: Plug, color: "text-muted" },
  "experimental-target": { icon: FlaskConical, color: "text-warn" },
  "no-changes": { icon: CircleCheck, color: "text-ok" },
};

/** Notes carry a code and params; the text always comes from the catalog. */
export function useNoteText() {
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

function Line({ icon: Icon, color, bold, text, hint }: Look & { bold: boolean; text: ReactNode; hint?: ReactNode }) {
  return (
    <li className="flex items-start gap-2.5">
      <Icon className={`mt-px size-5 shrink-0 ${color}`} aria-hidden strokeWidth={2.25} />
      <span>
        <span className={bold ? "font-bold" : ""}>{text}</span>
        {hint && <span className="mt-0.5 block text-xs leading-snug text-muted">{hint}</span>}
      </span>
    </li>
  );
}

/** Where the file comes from: version, build, system. */
function SavedIn({ file }: { file: FileInfo }) {
  const { t } = useTranslation();
  const v = file.version;
  if (!v) return null;
  const parts = [
    v.name ? t("file.version", { name: v.name }) : t("file.versionPlain"),
    `${versionNumber(v)}${v.release ? "" : ` ${t("file.beta")}`}`,
    t("file.build", { build: v.build }),
    v.platform !== "unknown" ? t(`platform.${v.platform}`) : null,
  ].filter(Boolean);
  return (
    <p className="mb-2 text-xs text-muted">
      {t("report.saved")} {parts.join(" · ")}
    </p>
  );
}

export function Report({ id, file, report, year }: { id: string; file: FileInfo; report?: Preflight; year: string }) {
  const { t } = useTranslation();
  const noteText = useNoteText();
  const status = useAppStore((s) => s.targets.find((x) => x.major === report?.target)?.status);

  let body: ReactNode;
  if (file.problem) {
    const reason = t(`problem.${file.problem}`);
    body = (
      <>
        <ul className="flex flex-col gap-1.5">
          <Line
            icon={isIgnored(file) ? CircleMinus : OctagonX}
            color={isIgnored(file) ? "text-muted" : "text-danger"}
            bold
            text={reason.charAt(0).toUpperCase() + reason.slice(1)}
            hint={t(`problemHint.${file.problem}`, { defaultValue: "" }) || undefined}
          />
        </ul>
        {!isIgnored(file) && (
          <p className="mt-2">
            <DetailsButton fileId={file.id} name={file.name} />
          </p>
        )}
      </>
    );
  } else if (!report) {
    body = <p className="text-muted">{t("report.loading")}</p>;
  } else {
    // The experimental note is last; a file with nothing else to say loses nothing.
    const experimental = report.notes.filter((n) => n.code === "experimental-target");
    const rest = report.notes.filter((n) => n.code !== "experimental-target");
    const notes: Note[] = [
      ...(rest.length ? rest : report.blocked ? [] : [{ severity: "info" as const, code: "no-changes", params: { year } }]),
      ...experimental,
    ];
    body = (
      <>
        <SavedIn file={file} />
        <h3 className="mb-2 flex flex-wrap items-center gap-x-2.5 gap-y-1 text-base font-extrabold">
          {t("report.for")} {t("target.ae", { name: year })}
          {status && <StatusBadge status={status} pill />}
        </h3>
        <ul className="mb-2 flex flex-col gap-1.5">
          {notes.map((note) => {
            const look = BY_CODE[note.code] ?? BY_SEVERITY[note.severity];
            const { text, hint } = noteText(note);
            return <Line key={note.code} {...look} bold={note.severity !== "info"} text={text} hint={hint} />;
          })}
        </ul>
        {report.blocked && !notes.some((n) => n.code === "not-a-downgrade" || n.code === "target-not-older") && (
          <p className="mb-2">
            <DetailsButton fileId={file.id} name={file.name} />
          </p>
        )}
        <p className="flex items-center gap-2 text-xs text-muted">
          <ShieldCheck className="size-3.5 shrink-0" aria-hidden />
          {t("report.safe")}
        </p>
      </>
    );
  }

  return (
    <motion.div
      id={id}
      initial={{ height: 0, opacity: 0 }}
      animate={{ height: "auto", opacity: 1 }}
      exit={{ height: 0, opacity: 0 }}
      transition={{ duration: 0.26, ease: [0.22, 1, 0.36, 1] }}
      className="overflow-hidden"
    >
      <div className="mt-1 mb-3 ml-[68px] rounded-md bg-raised/60 px-4 py-3 text-sm max-[820px]:ml-2">{body}</div>
    </motion.div>
  );
}
