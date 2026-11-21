// The window buttons of the app's own title bar on Windows (tauri.windows.conf.json turns the
// system frame off there). Nothing happens outside Tauri (the browser preview).
import { isTauri } from "./ipc-tauri";

export async function windowAction(action: "minimize" | "toggleMaximize" | "close") {
  if (!isTauri()) return;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await getCurrentWindow()[action]();
}

/** Calls `onChange` with whether the window is maximized now and after every resize. */
export async function watchMaximized(onChange: (maximized: boolean) => void): Promise<() => void> {
  if (!isTauri()) return () => {};
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  const win = getCurrentWindow();
  onChange(await win.isMaximized());
  return win.onResized(async () => onChange(await win.isMaximized()));
}
