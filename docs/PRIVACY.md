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

The only network request is the **update check**. It downloads `latest.json`, a small public file attached to the project's latest release on GitHub. The file holds the newest version number, its release notes and the download links. The request carries no information about you or your files: no user id, no file names, no list of what you converted. As with any request, GitHub sees your IP address and the program making the request.

- It runs in the background, at most once a day. **Settings > Check now** runs it on demand.
- You can turn it off: **Settings > Check for updates automatically**.
- Beta versions read the same kind of file from a separate "Update feed for beta builds" release.

**Installing an update** happens only when you click **Update**. The app then downloads the new version from the same GitHub release and checks its signature against the public key built into the app (`plugins.updater.pubkey` in `tauri.conf.json`) before installing it. A file that does not match the signature is not installed. Copies the app cannot update itself (a `.deb` package, an app started from the macOS disk image, a build from source) open the release page in your browser instead.

The app opens a link in your web browser only when you click one, such as the project page on GitHub or the page about supporting the project. Fonts and images are bundled in the app. It loads nothing from the internet to draw its window.

## Settings

The app remembers your choices in a small `settings.json` file in the app's per-user data folder. Which folder that is depends on your system. The portable Windows version keeps it next to its `.exe` instead, as `AEP Downgrader settings.json` (or in the data folder if it cannot write there). It holds:

- language, theme and accent color
- where to save copies (next to the original, or a chosen folder) and that folder's path
- the file name pattern
- what to do when a file name already exists
- whether to check for updates
- the version that ran last (to show what is new after an update) and the time of the last update check

It does not hold a list of your files. Delete the file to reset the app.

## Diagnostic reports

The **Activity log** (the pulse icon at the top of the window) lists what the app did in this session: files added, what was detected in them, the checks before converting, and the outcome of each conversion. It lives in memory and is gone when you close the app.

**Copy log** copies the log as text, to paste into a bug report. It contains the app version, the operating system and processor type, and the log. File names and folders are hidden unless you tick **Include full paths**, which is off every time the app starts. You see the text before you paste it anywhere. The app never sends reports by itself.

## The website

[aepdowngrader.com](https://aepdowngrader.com) is a set of static pages hosted on GitHub Pages. It has no analytics, no counters, no cookies and no ads, and loads nothing from other sites. Like any web host, GitHub sees the IP address of visitors. The download buttons lead to the files of the GitHub release.

## Donations

Donations go through [Boosty](https://boosty.to/aepdowngrader) and [Tribute](https://web.tribute.tg/d/UNE). They are separate services with their own terms and privacy policies, and your payment details go to them, not to this project. The project sees only what these services show their creators, such as the amount and the name and message you choose to leave. The app knows nothing about donations.

## Command line

`aep` makes no network requests and keeps no settings. It reads the files you name and writes the output file you name.

## Reporting a problem file

If a file fails and you cannot share it because it is confidential, say so in the issue and do not attach it. Project files contain the paths of your footage. See [CONTRIBUTING.md](../CONTRIBUTING.md#sending-a-file-privately).
