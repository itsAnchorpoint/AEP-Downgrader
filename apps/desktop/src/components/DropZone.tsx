import { Download } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../store";
import { Lockup } from "./Lockup";
import { PrivacyLine } from "./ui";

// Rings around the 104 px circle. The outer one reaches 108 px from the centre (56 px past the
// circle); index.css (.drop-circle) gives it room for that and shrinks it in a short drop zone.
const RING_SIZES = ["size-[140px] -m-[70px] opacity-55", "size-[178px] -m-[89px] opacity-30", "size-[216px] -m-[108px] opacity-15"];
const EXTENSIONS = [".aep", ".aet", ".ffx"];

export function DropZone() {
  const { t } = useTranslation();
  const dragging = useAppStore((s) => s.dragging);
  const browse = useAppStore((s) => s.browse);

  return (
    <div className="flex h-full flex-col gap-3">
      <Lockup />

      <button
        type="button"
        onClick={() => void browse()}
        className={`drop-zone group relative flex min-h-0 flex-1 flex-col items-center justify-center gap-1 overflow-hidden rounded-xl surface p-6 text-center transition-[scale] duration-200 ease-out-quint active:scale-[0.995] ${
          dragging ? "drop-target-active" : ""
        }`}
      >
        <span className="drop-circle relative grid size-26 shrink-0 place-items-center" aria-hidden>
          {RING_SIZES.map((size, i) => (
            <span
              key={size}
              className={`ring absolute top-1/2 left-1/2 rounded-full border-2 border-accent-text ${size}`}
              style={{ animationDelay: `${i * 0.5}s` }}
            />
          ))}
          <span
            className={`relative z-10 grid size-26 place-items-center rounded-full fill-accent transition-transform duration-300 ease-pop group-hover:scale-110 ${
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
        <span className="drop-chips relative mt-3.5 flex gap-1.5" aria-hidden>
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
