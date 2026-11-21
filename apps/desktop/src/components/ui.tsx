import { Check, FlaskConical, Lock } from "lucide-react";
import type { ButtonHTMLAttributes, ReactNode } from "react";
import { useTranslation } from "react-i18next";

type Variant = "primary" | "secondary" | "ghost" | "link";

const VARIANTS: Record<Variant, string> = {
  primary:
    "rounded-full px-4 fill-strong font-extrabold hover:scale-[1.03] active:scale-[0.96] disabled:opacity-40 disabled:hover:scale-100",
  secondary:
    "rounded-full px-4 shadow-outline text-fg hover:bg-hover active:bg-press active:scale-[0.96] disabled:opacity-40",
  ghost: "rounded-md text-muted hover:text-fg hover:bg-hover active:bg-press disabled:opacity-40",
  link: "rounded-sm px-1 underline decoration-2 underline-offset-4 hover:bg-hover",
};

export function Button({
  variant = "secondary",
  className = "",
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: Variant }) {
  return (
    <button
      type="button"
      className={`inline-flex min-h-9 items-center justify-center gap-2 text-sm font-bold whitespace-nowrap transition-[translate,scale,background-color,color,opacity] duration-150 disabled:cursor-not-allowed ${VARIANTS[variant]} ${className}`}
      {...props}
    />
  );
}

/** Square button that only shows an icon; the label is mandatory for screen readers. */
export function IconButton({
  label,
  children,
  className = "",
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & { label: string; children: ReactNode }) {
  return (
    <Button variant="ghost" aria-label={label} title={label} className={`size-9 ${className}`} {...props}>
      {children}
    </Button>
  );
}

/** `pill` is an outlined chip (report title); `quiet` mutes it (target tiles that are not chosen). */
export function StatusBadge({
  status,
  pill = false,
  quiet = false,
}: {
  status: "verified" | "experimental";
  pill?: boolean;
  quiet?: boolean;
}) {
  const { t } = useTranslation();
  const Icon = status === "verified" ? Check : FlaskConical;
  const color = quiet ? "text-muted" : status === "verified" ? "text-ok" : "text-warn";
  const shape = pill ? "rounded-full px-2.5 py-0.5 shadow-[inset_0_0_0_1.5px_currentColor]" : "";
  return (
    <span className={`inline-flex items-center gap-1 text-xs font-bold ${color} ${shape}`}>
      <Icon className="size-3" aria-hidden strokeWidth={3} />
      {/* A quiet badge in a narrow window keeps only its icon; the tile's tooltip says the word. */}
      <span className={quiet ? "max-[820px]:sr-only" : ""}>{t(`target.${status}`)}</span>
    </span>
  );
}

export function Segmented<T extends string>({
  value,
  onChange,
  options,
  label,
  disabled = false,
}: {
  value: T;
  onChange: (value: T) => void;
  options: { value: T; label: string; leading?: ReactNode }[];
  label: string;
  disabled?: boolean;
}) {
  return (
    <div role="group" aria-label={label} className="flex flex-wrap justify-end gap-1.5">
      {options.map((o) => {
        const on = o.value === value;
        return (
          <button
            key={o.value}
            type="button"
            aria-pressed={on}
            disabled={disabled}
            onClick={() => onChange(o.value)}
            className={`inline-flex items-center gap-2 rounded-full px-4 py-1.5 text-sm font-bold transition-[translate,scale,background-color] duration-200 active:scale-[0.96] disabled:cursor-not-allowed disabled:opacity-40 disabled:active:scale-100 ${
              on ? "fill-selected" : "text-muted shadow-[inset_0_0_0_1.5px_var(--color-line)] enabled:hover:bg-hover enabled:hover:text-fg"
            }`}
          >
            {o.leading}
            {o.label}
          </button>
        );
      })}
    </div>
  );
}

/** The product's headline promise, shown on the first screen and while converting. */
/** The footer of the start and progress screens: the privacy promise, and what the app is for. */
export function PrivacyLine() {
  const { t } = useTranslation();
  return (
    <div className="flex flex-wrap items-center gap-x-2.5 gap-y-1 px-0.5 text-sm font-bold max-[820px]:text-xs">
      <Lock className="size-4.5 shrink-0 text-accent-text" aria-hidden strokeWidth={2.5} />
      <span>{t("drop.privacy")}</span>
      <span className="ml-auto font-medium text-muted max-[820px]:hidden">{t("drop.tagline")}</span>
    </div>
  );
}
