import { Check, FlaskConical, Lock } from "lucide-react";
import type { ButtonHTMLAttributes, ReactNode } from "react";
import { useTranslation } from "react-i18next";

type Variant = "primary" | "secondary" | "ghost" | "link";

const VARIANTS: Record<Variant, string> = {
  primary:
    "rounded-full px-4 bg-strong text-on-strong font-extrabold hover:scale-[1.03] active:scale-[0.97] disabled:opacity-40 disabled:hover:scale-100",
  secondary:
    "rounded-full px-4 shadow-outline text-fg hover:bg-hover active:bg-press active:scale-[0.97] disabled:opacity-40",
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

/** `pill` is a filled chip (report title), otherwise plain colored text (target tiles). */
export function StatusBadge({ status, pill = false }: { status: "verified" | "experimental"; pill?: boolean }) {
  const { t } = useTranslation();
  const Icon = status === "verified" ? Check : FlaskConical;
  const tone = pill
    ? status === "verified"
      ? "rounded-full bg-accent px-2.5 py-0.5 text-accent-fg"
      : "rounded-full bg-warn-fill px-2.5 py-0.5 text-ink-black"
    : status === "verified"
      ? "text-ok"
      : "text-warn";
  return (
    <span className={`inline-flex items-center gap-1 text-xs font-bold ${tone}`}>
      <Icon className="size-3" aria-hidden strokeWidth={3} />
      {t(`target.${status}`)}
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
            className={`inline-flex items-center gap-2 rounded-full px-4 py-1.5 text-sm font-bold transition-[translate,scale,background-color] duration-200 active:scale-95 disabled:cursor-not-allowed disabled:opacity-40 disabled:active:scale-100 ${
              on ? "bg-strong text-on-strong" : "shadow-outline enabled:hover:bg-hover"
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
export function PrivacyLine() {
  const { t } = useTranslation();
  return (
    <div className="flex flex-wrap items-center gap-x-2.5 gap-y-1 px-0.5 text-sm font-bold max-[820px]:text-xs">
      <Lock className="size-4.5 shrink-0 text-accent-text" aria-hidden strokeWidth={2.5} />
      <span>{t("drop.privacy")}</span>
      <span className="ml-auto font-medium text-muted max-[820px]:hidden">{t("drop.privacySub")}</span>
    </div>
  );
}
