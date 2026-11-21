// Which system the app runs on, for the title bar and system names ("Show in Finder").
// In development, ?os=mac|windows|linux previews another system's title bar in the browser.
export type Platform = "mac" | "windows" | "linux";

const override = import.meta.env.DEV ? new URLSearchParams(window.location.search).get("os") : null;

export const platform: Platform =
  override === "mac" || override === "windows" || override === "linux"
    ? override
    : /Mac/i.test(navigator.platform)
      ? "mac"
      : /Win/i.test(navigator.platform)
        ? "windows"
        : "linux";

/** A shortcut as the system writes it: "⌘O" on macOS, "Ctrl+O" elsewhere. */
export function shortcut(key: string): string {
  return platform === "mac" ? `⌘${key === "Enter" ? "↩" : key}` : `Ctrl+${key}`;
}
