# Privacy

AEP Downgrader works entirely on your computer. Your projects are not uploaded anywhere. There is no account and no telemetry.

The source code is open ([GPLv3](../LICENSE)), so you do not have to take this page on trust. The places to look:

- `crates/aep-core`: the conversion. It has no dependencies and does no file or network access; it turns bytes into bytes.
- `apps/desktop/src-tauri`: the app's native side. It reads and writes the files you choose.
- `apps/desktop/src-tauri/capabilities` and `tauri.conf.json`: what the app window is allowed to do, including a content security policy that limits the window to the app's own files and local calls.

## What the app reads

- The files you drop in or choose. To identify a file it reads the first part of it (the header). To convert a file it reads the whole file into memory.
- When you drop a folder, it lists the `.aep`, `.aet` and `.ffx` files directly inside that folder (not in subfolders).
- Its own settings (see below).

It does not scan your disk, and it does not read footage or any other file that a project refers to.

## What the app writes

- **The converted copy**, in the folder you chose (next to the original by default). The copy is written under a temporary name in that folder and then renamed. If writing fails, the temporary file is removed.
- **Its settings file** (see below).

The original file is never modified. If a copy would end up replacing the original, nothing is written.

## What the app sends

Nothing about your files. Not their names, not their content, not their version.

The only network request is the **update check**. It asks GitHub for the latest release number of this project. It does not send your files, a user id or a list of what you converted. As with any request, GitHub sees your IP address and the program making the request.

- It runs in the background, at most once a day.
- You can turn it off: **Settings > Check for updates automatically**.
- The update mechanism is still being finished in the development branch. This page will say so when it changes.

The app opens a link in your web browser only when you click one (the project page on GitHub). Fonts and images are bundled in the app. It loads nothing from the internet to draw its window.

## Settings

The app remembers your choices in a small `settings.json` file in the app's per-user data folder. Which folder that is depends on your system. It holds:

- language, theme and accent colour
- where to save copies (next to the original, or a chosen folder) and that folder's path
- the file name pattern
- what to do when a file name already exists
- whether to check for updates

It does not hold a list of your files. Delete the file to reset the app.

## Diagnostic reports

A diagnostic report, to attach to a bug report, is being added. It is meant to contain the app version, the operating system and the outcome of conversions, with file paths redacted by default, and you will be able to read it before sharing it. This page will be updated with the exact contents when it ships. The app does not send reports by itself.

## Command line

`aep` makes no network requests and keeps no settings. It reads the files you name and writes the output file you name.

## Reporting a problem file

If a file fails and you cannot share it because it is confidential, say so in the issue and do not attach it. Project files contain the paths of your footage. See [CONTRIBUTING.md](../CONTRIBUTING.md#sending-a-file-privately).
