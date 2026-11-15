import { Download } from "lucide-react";
import { motion } from "motion/react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../store";
import { PrivacyLine } from "./ui";

const RING_SIZES = ["size-[150px] -m-[75px] opacity-55", "size-[190px] -m-[95px] opacity-30", "size-[230px] -m-[115px] opacity-15"];
const EXTENSIONS = [".aep", ".aet", ".ffx"];

export function DropZone() {
  const { t } = useTranslation();
  const dragging = useAppStore((s) => s.dragging);
  const browse = useAppStore((s) => s.browse);

  return (
    <div className="flex h-full flex-col gap-3">
      <motion.h1
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, ease: [0.2, 0.9, 0.2, 1] }}
        className="text-hero leading-[1.02] font-black tracking-display [word-spacing:0.04em] text-balance"
      >
        {t("drop.heroBefore")}{" "}
        <motion.mark
          initial={{ opacity: 0, rotate: -8, scale: 0.7 }}
          animate={{ opacity: 1, rotate: -2, scale: 1 }}
          transition={{ delay: 0.25, type: "spring", stiffness: 380, damping: 14 }}
          className="inline-block rounded-[0.12em] bg-accent px-[0.12em] text-accent-fg"
        >
          {t("drop.heroMark")}
        </motion.mark>{" "}
        {t("drop.heroAfter")}
      </motion.h1>

      <button
        type="button"
        onClick={() => void browse()}
        className={`drop-zone group relative flex min-h-0 flex-1 flex-col items-center justify-center gap-1 overflow-hidden rounded-xl bg-surface p-6 text-center transition-[translate,scale] duration-300 ease-out-quint hover:scale-[1.008] active:scale-[0.99] ${
          dragging ? "shadow-outline-accent" : "shadow-outline"
        }`}
      >
        <span className="relative mb-12 grid size-26 place-items-center max-[820px]:mb-10" aria-hidden>
          {RING_SIZES.map((size, i) => (
            <span
              key={size}
              className={`ring absolute top-1/2 left-1/2 rounded-full border-2 border-accent-text ${size}`}
              style={{ animationDelay: `${i * 0.5}s` }}
            />
          ))}
          <span
            className={`relative z-10 grid size-26 place-items-center rounded-full bg-accent text-accent-fg transition-transform duration-300 ease-pop group-hover:scale-110 ${
              dragging ? "scale-125" : ""
            }`}
          >
            <Download className="bob size-12" strokeWidth={2.75} />
          </span>
        </span>
        <span className="relative text-[clamp(1.375rem,3.6vw,2.125rem)] font-extrabold tracking-display">
          {dragging ? t("drop.over") : t("drop.title")}
        </span>
        <span className="relative text-base text-muted">{t("drop.sub")}</span>
        <span className="relative mt-3.5 flex gap-1.5" aria-hidden>
          {EXTENSIONS.map((ext) => (
            <i
              key={ext}
              className="rounded-full px-3 py-0.5 font-mono text-sm font-bold not-italic shadow-outline"
            >
              {ext}
            </i>
          ))}
        </span>
      </button>

      <PrivacyLine />
    </div>
  );
}
