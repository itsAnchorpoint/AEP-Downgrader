//! NSIS plug-in that draws the window of the AEP Downgrader installer and uninstaller.
//!
//! The installer script (`../installer.nsi`) keeps NSIS's pages and logic and calls this
//! plug-in to show each page: `aepd_ui::Init` takes over the installer window, `Page` shows a
//! page, `Status` updates the progress line from the install thread and `Result` reports the
//! user's choice on the last page. Everything is drawn in software (tiny-skia, rustybuzz), so
//! the window looks the same on every Windows version and under Wine.

pub mod paint;
pub mod text;
pub mod ui;

#[cfg(windows)]
mod nsis;
#[cfg(windows)]
mod win;

#[cfg(test)]
mod tests {
    use crate::ui::{Kind, PageSpec, Strings, Ui};
    use tiny_skia::Pixmap;

    fn strings(ru: bool) -> Strings {
        Strings {
            product: "AEP Downgrader".into(),
            version: "2.0.0".into(),
            footer_lead: if ru {
                "Всё остаётся на этом компьютере"
            } else {
                "Everything stays on this computer"
            }
            .into(),
            footer_sub: if ru {
                "Без аккаунта, без загрузки, без телеметрии."
            } else {
                "No account. No upload. No telemetry."
            }
            .into(),
        }
    }

    fn page(
        kind: Kind,
        title: &str,
        sub: &str,
        primary: &str,
        secondary: &str,
        check: &str,
        on: bool,
    ) -> PageSpec {
        PageSpec {
            kind,
            title: title.into(),
            sub: sub.into(),
            primary: primary.into(),
            secondary: secondary.into(),
            check: check.into(),
            check_on: on,
        }
    }

    /// Renders every page into $AEPD_PREVIEW (a directory) for design review:
    /// `AEPD_PREVIEW=/tmp/x cargo test --release preview -- --ignored`
    #[test]
    #[ignore]
    fn preview() {
        let dir = std::env::var("AEPD_PREVIEW").expect("AEPD_PREVIEW");
        let scale: f32 = std::env::var("AEPD_SCALE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.5);
        let pages = [
            ("1-welcome", false, page(Kind::Welcome, "Install in [one click]", "Open After Effects projects, templates and presets in older versions. **Free and open source.**", "Install", "Change folder", "", false)),
            ("2-installing", false, page(Kind::Progress, "Installing", "AEP Downgrader 2.0.0 goes to your user folder. This takes a few seconds.", "Copying files…", "", "", false)),
            ("3-done", false, page(Kind::Finish, "All [set]", "Find AEP Downgrader in the Start menu.", "Open AEP Downgrader", "Close", "Add a desktop shortcut", true)),
            ("4-update", false, page(Kind::Progress, "Updating to [2.0.1]", "The app opens again by itself when it is done.", "Replacing the app…", "", "", false)),
            ("5-uninstall", false, page(Kind::Confirm, "Sorry to see you [go]", "This removes AEP Downgrader. Your After Effects files stay where they are.", "Remove", "Keep it", "Also delete settings and history", false)),
            ("6-welcome-ru", true, page(Kind::Welcome, "Установка в [один клик]", "Открывайте проекты, шаблоны и пресеты After Effects в старых версиях. **Бесплатно и с открытым кодом.**", "Установить", "Выбрать папку", "", false)),
            ("7-folder", false, page(Kind::Welcome, "Install in [one click]", "Open After Effects projects, templates and presets in older versions. **Free and open source.**", "Install", "Change folder", "", false)),
            ("8-error", false, page(Kind::Error, "Something went [wrong]", "The installation stopped before it finished. Close this window and run the installer again.", "Close", "", "", false)),
            ("9-removed", false, page(Kind::Done, "All [done]", "AEP Downgrader is removed. Thanks for giving it a try.", "Close", "", "", false)),
        ];
        for (name, ru, spec) in pages {
            let mut ui = Ui::new(scale, strings(ru), false);
            let progress = name == "2-installing" || name == "4-update";
            ui.set_page(spec);
            if progress {
                ui.progress = if name == "2-installing" { 0.64 } else { 0.38 };
            }
            if name == "1-welcome" {
                ui.hover = Some(crate::ui::Hit::Primary);
            }
            if name == "7-folder" {
                ui.set_link(r"D:\Work\Tools\Motion design\Plug-ins\AEP Downgrader".into());
            }
            for _ in 0..120 {
                ui.tick(1.0 / 60.0);
            }
            let (w, h) = ui.pixel_size();
            let mut out = Pixmap::new(w, h).unwrap();
            ui.render(&mut out);
            out.save_png(format!("{dir}/{name}.png")).unwrap();
        }
    }
}
