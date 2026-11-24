# AEP Downgrader tester kit

Thank you for helping! AEP Downgrader rewrites After Effects project files so an older After Effects can open them. To get the rules right, we compare files that After Effects itself wrote with the copies it makes through **File > Save a Copy As > (older version)**. The maintainer only owns After Effects 2024, so we need your version and your Mac or PC.

These scripts build a small **test project** (gray solids, shapes, placeholder text, effects, nothing personal, no footage, no file paths from your computer), save it, and let you save older-version copies of it. It takes about 10-15 minutes.

You do not need to know scripting. You only run three scripts from the File menu.

## 0. One-time setup: allow scripts to write files

1. Open After Effects.
2. Windows: **Edit > Preferences > Scripting & Expressions**. Mac: **After Effects > Settings > Scripting & Expressions**.
3. Tick **Allow Scripts to Write Files and Access Network**. Click OK.

If this is off, each script tells you so and stops without doing anything.

Also: start with an empty After Effects (**File > Close Project**, do not save). The scripts never touch an open project.

Create one empty folder for the results, for example `AEPD_results` on your Desktop. Use it for every step below.

## 1. Build the test project: `make_test_project.jsx`

1. **File > Scripts > Run Script File...** and choose `make_test_project.jsx`.
2. Choose your results folder when asked.
3. Wait. Building the "FX all" comp applies hundreds of effects and can take a few minutes. After Effects may look busy; that is normal. If it is stuck for over 15 minutes, force-quit and tell us (see "If something goes wrong").
4. One message appears at the end. It shows the project path, how many problems were logged, and **which older-version "Save a Copy As" targets your After Effects has**. Write them down or keep the message.

You now have `AEPD_test_<version>_<win|mac>.aep` and `make_test_project_log_<version>_<win|mac>.txt` in the folder. The project stays open.

Tip: if the run is very slow on your computer, open the script in a text editor and change `var INCLUDE_ALL_EFFECTS = true;` to `false` at the top. Tell us you did that.

## 2. Save copies for older versions ("Save a Copy As")

With the test project still open, for **every** older version that appears in the File menu:

1. **File > Save > Save a Copy As >** pick the entry, for example **Save a Copy As 24.x...** (the exact list depends on your version; the scripts looked for 26.x, 25.x, 24.x, 23.x, 22.x, 18.x and 17.x).
2. Name the file `AEPD_test_<saved-from>_as_<target>.aep`.
   `<saved-from>` is your version as in the project name (for example `25.2_win`), `<target>` is the version you picked (for example `24.x` becomes `24`).
   Example: `AEPD_test_25.2_win_as_24.aep`, `AEPD_test_25.2_win_as_23.aep`.
3. Save into the **same results folder**. If After Effects shows a warning that some features are not supported in the older version, just confirm it. Write down the warnings text if you can (a screenshot or a copy in a text file named `warnings_<target>.txt` is great).

Do all available targets. If none is available, skip this step and say so when you send the results.

## 3. Save one animation preset and one template

1. Open the comp **FX core** (in the folder "04 Effects"). Pick any layer and select one or two of its effects (for example Gaussian Blur and Fractal Noise, the ones that have keyframes).
2. **Animation > Save Animation Preset...** and save it into the results folder as `AEPD_preset_<your-version>.ffx`.
3. For the template: **File > Save As > Save a Copy...** (or **Save As...**), and in the "Save as type" list choose the template type (After Effects Template Project, `.aet`). Name it `AEPD_template_<your-version>.aet` in the results folder. If your version has no such choice, skip the template and tell us.

## 4. Check that everything opens: `verify_open.jsx`

1. **Close the project without saving** (File > Close Project, "Don't Save").
2. **File > Scripts > Run Script File...** and choose `verify_open.jsx`.
3. Pick the results folder. The script opens every `.aep` and `.aet` in it one after another (without saving anything) and applies every `.ffx` preset, then writes `verify_report_<version>_<win|mac>.json`.
4. One message shows how many opened and how many failed. Failures are normal for files of a newer version; just keep the report.

## 5. List your effects: `dump_effects.jsx`

1. **File > Scripts > Run Script File...** and choose `dump_effects.jsx`.
2. Pick the results folder. It writes `effects_<version>_<win|mac>.json`, the list of effects installed in your After Effects (this tells us which effects exist in your version).

## What to send back

Zip the **whole results folder** (right-click > Compress / Send to compressed folder) and send it to the maintainer. It should contain:

- `AEPD_test_<version>_<os>.aep`
- all `AEPD_test_..._as_<target>.aep` copies
- the `.ffx` preset and the `.aet` template (if you managed)
- `make_test_project_log_*.txt`, `verify_report_*.json`, `effects_*.json`
- optional: screenshots or notes of warnings

Also tell us: your After Effects version (Help > About), Windows or Mac, and any installed third-party plugins you know about.

## If something goes wrong

- A script says "problems logged" or shows errors: that is OK. Many features do not exist in every version, and each one is skipped separately. **Send the log anyway.**
- A script does nothing: check the setup in step 0, and make sure no project is open.
- After Effects hangs on one effect in "FX all": force-quit, run again with `INCLUDE_ALL_EFFECTS = false`, and tell us the last thing you saw.
- No "Save a Copy As" entries appear: tell us; maybe your version has them with another wording. We will adapt.

Nothing in these scripts uploads anything. Files only go where you send them.
