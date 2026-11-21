import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { Ipc } from "./ipc";
import type { DownloadEvent, ProgressEvent } from "./ipc-types";

export { isTauri };

const FILTERS = [{ name: "After Effects", extensions: ["aep", "aet", "ffx"] }];

export function createTauriIpc(): Ipc {
  return {
    inspectPaths: (paths) => invoke("inspect_paths", { paths }),
    listTargets: () => invoke("list_targets"),
    preflight: (path, target) => invoke("preflight", { path, target }),
    convert: (jobs, options) => invoke("convert", { jobs, options }),
    cancelConversion: () => invoke("cancel_conversion"),
    revealInFolder: (path) => invoke("reveal_in_folder", { path }),
    onProgress: (handler) => listen<ProgressEvent>("convert://progress", (e) => handler(e.payload)),
    async pickFiles() {
      const picked = await open({ multiple: true, filters: FILTERS });
      return picked ?? [];
    },
    async pickFolder() {
      const picked = await open({ directory: true });
      return picked ?? null;
    },
    onFileDrop: (handlers) =>
      getCurrentWebview().onDragDropEvent(({ payload }) => {
        if (payload.type === "enter" || payload.type === "over") handlers.hover(true);
        else if (payload.type === "leave") handlers.hover(false);
        else {
          handlers.hover(false);
          handlers.drop(payload.paths);
        }
      }),
    openUrl: (url) => openUrl(url),
    systemInfo: () => invoke("system_info"),
    appInfo: () => invoke("app_info"),
    checkUpdate: () => invoke("check_update"),
    installUpdate(onEvent) {
      const channel = new Channel<DownloadEvent>();
      channel.onmessage = onEvent;
      return invoke("install_update", { onEvent: channel });
    },
    restartApp: () => invoke("restart_app"),
  };
}
