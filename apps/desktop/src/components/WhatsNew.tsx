import * as Dialog from "@radix-ui/react-dialog";
import { Check, Heart, type LucideIcon, Plus, RefreshCw, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import Markdown, { type Components } from "react-markdown";
import { useTranslation } from "react-i18next";
import changelog from "../content/CHANGELOG.md?raw";
import { ipc } from "../lib/ipc";
import { REPO_URL } from "../lib/links";
import { useAppStore } from "../store";
import { Button, IconButton } from "./ui";

// react-markdown builds React elements and skips raw HTML, so the text is never injected as markup.
const SECTION_TAGS: Record<string, { icon: LucideIcon; tone: string }> = {
  Added: { icon: Plus, tone: "bg-accent text-accent-fg" },
  Changed: { icon: RefreshCw, tone: "bg-accent text-accent-fg" },
  Fixed: { icon: Check, tone: "bg-warn-fill text-ink-black" },
};

const components: Components = {
  h1: () => null, // the dialog title replaces the file's own heading
  h3: ({ children }) => {
    const tag = SECTION_TAGS[String(children)];
    return (
      <h3 className="mt-4 mb-1 flex items-center gap-2 text-sm font-black tracking-[0.12em] uppercase">
        {tag && (
          <span className={`grid size-5.5 place-items-center rounded-[7px] ${tag.tone}`} aria-hidden>
            <tag.icon className="size-3.5" strokeWidth={3} />
          </span>
        )}
        {children}
      </h3>
    );
  },
  p: (props) => <p className="my-1 text-sm text-muted" {...props} />,
  ul: (props) => <ul {...props} />,
  li: (props) => (
    <li
      className="relative border-b border-line py-2 pl-5.5 text-base font-semibold tracking-tight before:absolute before:top-[1.1em] before:left-0.5 before:size-2 before:rotate-45 before:rounded-[2px] before:bg-fg"
      {...props}
    />
  ),
  code: (props) => <code className="rounded-sm bg-raised px-1 font-mono text-xs" {...props} />,
  a: ({ href, children }) => (
    <a
      href={href}
      className="font-bold underline decoration-2 underline-offset-4"
      onClick={(e) => {
        e.preventDefault();
        if (href?.startsWith("https://")) void ipc.openUrl(href);
      }}
    >
      {children}
    </a>
  ),
};

export function WhatsNew() {
  const { t } = useTranslation();
  const open = useAppStore((s) => s.whatsNewOpen);
  const setOpen = useAppStore((s) => s.setWhatsNewOpen);
  const update = useAppStore((s) => s.update);
  const installUpdate = useAppStore((s) => s.installUpdate);

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
      <AnimatePresence>
        {open && (
          <Dialog.Portal forceMount>
            <Dialog.Overlay asChild>
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="fixed inset-0 z-40 bg-scrim"
              />
            </Dialog.Overlay>
            <div className="fixed inset-0 z-50 flex items-center justify-center p-6">
              <Dialog.Content asChild aria-describedby={undefined}>
                <motion.div
                  initial={{ opacity: 0, y: 14, scale: 0.97 }}
                  animate={{ opacity: 1, y: 0, scale: 1 }}
                  exit={{ opacity: 0, y: 8 }}
                  transition={{ duration: 0.28, ease: [0.2, 0.8, 0.2, 1] }}
                  className="flex max-h-full w-full max-w-lg flex-col overflow-hidden rounded-xl bg-surface shadow-pop"
                >
                  <div className="flex items-start justify-between gap-3 px-6 pt-5">
                    <Dialog.Title className="rounded-full bg-accent px-2.5 py-0.5 text-xs font-black tracking-[0.12em] text-accent-fg uppercase">
                      {t("whatsNew.title")}
                    </Dialog.Title>
                    <Dialog.Close asChild>
                      <IconButton label={t("whatsNew.close")} className="rounded-full! shadow-outline">
                        <X className="size-4" aria-hidden />
                      </IconButton>
                    </Dialog.Close>
                  </div>
                  <div className="changelog min-h-0 flex-1 overflow-y-auto px-6 pt-2 pb-3">
                    <Markdown skipHtml components={components}>
                      {changelog}
                    </Markdown>
                  </div>
                  <div className="flex flex-wrap items-center gap-x-4 gap-y-2 px-6 pt-3 pb-5">
                    <Button variant="link" onClick={() => void ipc.openUrl(REPO_URL)}>
                      <Heart className="size-4 text-danger" aria-hidden />
                      {t("done.support")}
                    </Button>
                    <span className="mr-auto text-xs text-muted">{t("whatsNew.foot")}</span>
                    {update.status === "available" ? (
                      <Button variant="primary" className="h-11 text-base" onClick={() => void installUpdate()}>
                        {t("whatsNew.updateTo", { version: update.version })}
                      </Button>
                    ) : (
                      <Dialog.Close asChild>
                        <Button>{t("whatsNew.close")}</Button>
                      </Dialog.Close>
                    )}
                  </div>
                </motion.div>
              </Dialog.Content>
            </div>
          </Dialog.Portal>
        )}
      </AnimatePresence>
    </Dialog.Root>
  );
}
