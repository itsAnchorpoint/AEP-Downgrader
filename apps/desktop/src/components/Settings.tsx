import * as Switch from "@radix-ui/react-switch";
import { ArrowLeft, ExternalLink, Heart, ShieldCheck } from "lucide-react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { version } from "../../package.json";
import { ipc } from "../lib/ipc";
import { REPO_URL, supportUrl } from "../lib/links";
import { outputName } from "../lib/pattern";
import { ACCENTS, usesOwnAccent } from "../lib/settings";
import { useAppStore } from "../store";
import { Button, Segmented } from "./ui";

/** A titled group of rows, like a ruled ledger. */
function Group({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section>
      <h3 className="pb-1.5 text-sm font-bold text-muted">{title}</h3>
      <div className="border-t border-line">{children}</div>
    </section>
  );
}

function Row({ title, hint, children }: { title: ReactNode; hint?: ReactNode; children: ReactNode }) {
  return (
    <div className="flex min-h-[62px] items-center justify-between gap-4 border-b border-line py-3 max-[820px]:flex-wrap">
      <div className="min-w-0">
        <div className="text-base font-extrabold tracking-tight">{title}</div>
        {hint && <p className="mt-0.5 text-sm text-muted">{hint}</p>}
      </div>
      <div className="flex max-w-[62%] flex-wrap items-center justify-end gap-2 max-[820px]:ml-auto max-[820px]:max-w-full">{children}</div>
    </div>
  );
}

export function Settings() {
  const { t, i18n } = useTranslation();
  const { settings, targets, target } = useAppStore();
  const { updateSettings, chooseFolder, setView, checkUpdate, updateChecking, updateCheckResult } = useAppStore();
  const checkResult = {
    "up-to-date": t("settings.upToDate"),
    found: t("settings.updateFound"),
    failed: t("settings.checkFailed"),
  };
  const locked = usesOwnAccent(settings.theme);
  const year = targets.find((x) => x.major === target)?.name ?? "2024";
  // A copy with the original's name can't go next to the original (the engine refuses it).
  const sameName = settings.location === "next-to-source" && outputName(settings.pattern, "Intro_v05.aep", year) === "Intro_v05.aep";

  return (
    <div className="flex h-full flex-col gap-3">
      <div className="flex items-center gap-4">
        <Button onClick={() => setView("main")}>
          <ArrowLeft className="size-4" aria-hidden strokeWidth={2.5} />
          {t("header.back")}
        </Button>
        <h2 className="text-title leading-none font-extrabold tracking-display">{t("settings.title")}</h2>
      </div>

      <div className="scroll-fade flex min-h-0 flex-1 flex-col gap-6 overflow-x-hidden overflow-y-auto pb-6">
        <Group title={t("settings.groupFiles")}>
          <Row title={t("settings.save")}>
            <Segmented
              label={t("settings.save")}
              value={settings.location}
              onChange={(location) => updateSettings({ location })}
              options={[
                { value: "next-to-source", label: t("settings.saveNext") },
                { value: "folder", label: t("settings.saveFolder") },
              ]}
            />
          </Row>
          {settings.location === "folder" && (
            <Row
              title={
                <span className="block truncate font-mono text-sm" title={settings.folder ?? ""}>
                  {settings.folder ?? t("settings.noFolder")}
                </span>
              }
            >
              <Button onClick={() => void chooseFolder()}>{t("settings.choose")}</Button>
            </Row>
          )}
          <Row
            title={t("settings.pattern")}
            hint={
              <>
                {t("settings.patternHint")} {t("settings.preview")}:{" "}
                <span className="font-mono font-bold text-fg select-text">
                  {outputName(settings.pattern, "Intro_v05.aep", year)}
                </span>
                {sameName && <span className="mt-1 block font-bold text-warn">{t("settings.patternSame")}</span>}
              </>
            }
          >
            <input
              aria-label={t("settings.pattern")}
              value={settings.pattern}
              onChange={(e) => updateSettings({ pattern: e.target.value })}
              spellCheck={false}
              className="h-10 w-56 rounded-md bg-transparent px-3.5 font-mono text-sm font-semibold shadow-outline select-text focus:shadow-[inset_0_0_0_2px_var(--color-fg)] focus-visible:outline-none"
            />
          </Row>
          <Row title={t("settings.exists")} hint={settings.onExists === "overwrite" ? t("settings.overwriteHint") : undefined}>
            <Segmented
              label={t("settings.exists")}
              value={settings.onExists}
              onChange={(onExists) => updateSettings({ onExists })}
              options={[
                { value: "rename", label: t("settings.rename") },
                { value: "overwrite", label: t("settings.overwrite") },
                { value: "skip", label: t("settings.skip") },
              ]}
            />
          </Row>
          <Row
            title={<label htmlFor="auto-updates">{t("settings.updates")}</label>}
            hint={
              <>
                {t("settings.updatesSub")}{" "}
                <Button
                  variant="link"
                  className="min-h-0! px-0 align-baseline"
                  disabled={updateChecking}
                  onClick={() => void checkUpdate(true)}
                >
                  {updateChecking ? t("settings.checking") : t("settings.checkNow")}
                </Button>
                {updateCheckResult && !updateChecking && (
                  <span role="status" className="ml-2">
                    {checkResult[updateCheckResult]}
                  </span>
                )}
              </>
            }
          >
            <Switch.Root
              id="auto-updates"
              checked={settings.autoUpdates}
              onCheckedChange={(autoUpdates) => updateSettings({ autoUpdates })}
              className="relative h-7.5 w-13 shrink-0 rounded-full bg-toggle-off transition-colors data-[state=checked]:bg-strong"
            >
              <Switch.Thumb className="block size-5.5 translate-x-1 rounded-full bg-white transition-transform duration-200 ease-pop data-[state=checked]:translate-x-6.5 data-[state=checked]:bg-on-strong" />
            </Switch.Root>
          </Row>
        </Group>

        <Group title={t("settings.groupLook")}>
          <Row title={t("settings.language")}>
            <Segmented
              label={t("settings.language")}
              value={settings.language}
              onChange={(language) => updateSettings({ language })}
              options={[
                { value: "en", label: "English" },
                { value: "ru", label: "Русский" },
              ]}
            />
          </Row>
          <Row title={t("settings.theme")}>
            <Segmented
              label={t("settings.theme")}
              value={settings.theme}
              onChange={(theme) => updateSettings({ theme })}
              options={[
                { value: "system", label: t("settings.system") },
                { value: "light", label: t("settings.light") },
                { value: "dark", label: t("settings.dark") },
                { value: "ae", label: t("settings.ae") },
              ]}
            />
          </Row>
          <Row title={t("settings.accent")} hint={locked ? t("settings.accentLocked") : undefined}>
            {!locked && (
            <Segmented
              label={t("settings.accent")}
              disabled={locked}
              value={settings.accent}
              onChange={(accent) => updateSettings({ accent })}
              options={ACCENTS.map((accent) => ({
                value: accent,
                label: t(`settings.accents.${accent}`),
                leading: <i data-accent={accent} className="size-3.5 rounded-full bg-accent ring-2 ring-current/50" />,
              }))}
            />
            )}
          </Row>
        </Group>

        <section className="flex gap-4 rounded-lg surface p-5">
          <ShieldCheck className="size-9 shrink-0 text-accent-text" strokeWidth={2.25} aria-hidden />
          <div className="min-w-0">
            <h2 className="mb-1 text-xl leading-tight font-extrabold tracking-display">{t("privacy.title")}</h2>
            <p className="max-w-[62ch] text-sm text-muted">{t("privacy.body")}</p>
            <div className="mt-2.5 flex flex-wrap items-center justify-between gap-2">
              <div className="-ml-1 flex flex-wrap items-center gap-x-3">
                <Button variant="link" className="px-1 font-extrabold" onClick={() => void ipc.openUrl(REPO_URL)}>
                  {t("privacy.source")}
                  <ExternalLink className="size-3.5" aria-hidden />
                </Button>
                <Button
                  variant="link"
                  className="px-1 font-extrabold"
                  onClick={() => void ipc.openUrl(supportUrl(i18n.language))}
                >
                  <Heart className="size-3.5 text-accent-text" aria-hidden />
                  {t("done.support")}
                </Button>
              </div>
              <span className="font-mono text-xs text-muted">{t("settings.version", { version })}</span>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}
