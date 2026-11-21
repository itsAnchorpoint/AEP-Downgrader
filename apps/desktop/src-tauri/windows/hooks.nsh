; Included by installer.nsi (tauri.conf.json: bundle > windows > nsis > installerHooks), after the
; languages are loaded. Points NSIS at the window plug-in and holds the window's texts.
;
; Markup in titles: [words] sit on the lime plate. In subtitles: **words** are bold.

!addplugindir "${__FILEDIR__}\plugin"
!if ! /FileExists "${__FILEDIR__}\plugin\aepd_ui.dll"
  !error "The installer window plug-in is missing: run apps/desktop/scripts/installer-ui.mjs first"
!endif

; Footer, on every page
LangString aepdFooterLead ${LANG_ENGLISH} "Everything stays on this computer"
LangString aepdFooterLead ${LANG_RUSSIAN} "Всё остаётся на этом компьютере"
LangString aepdFooterSub ${LANG_ENGLISH} "No account, no uploads, no telemetry."
LangString aepdFooterSub ${LANG_RUSSIAN} "Без аккаунта, отправки файлов и телеметрии."

; Welcome: a first install
LangString aepdWelcomeTitle ${LANG_ENGLISH} "Install in [one click]"
LangString aepdWelcomeTitle ${LANG_RUSSIAN} "Установка в [один клик]"
LangString aepdWelcomeSub ${LANG_ENGLISH} "Open After Effects projects, templates and presets in older versions. **Free and open source.**"
LangString aepdWelcomeSub ${LANG_RUSSIAN} "Открывайте проекты, шаблоны и пресеты After Effects в старых версиях. **Бесплатно и с открытым кодом.**"
LangString aepdInstall ${LANG_ENGLISH} "Install"
LangString aepdInstall ${LANG_RUSSIAN} "Установить"
LangString aepdChangeFolder ${LANG_ENGLISH} "Change folder"
LangString aepdChangeFolder ${LANG_RUSSIAN} "Выбрать папку"

; Welcome: an older version is installed ($R0 is its version)
LangString aepdUpdateTitle ${LANG_ENGLISH} "Update to [${VERSION}]"
LangString aepdUpdateTitle ${LANG_RUSSIAN} "Обновление до [${VERSION}]"
LangString aepdUpdateSub ${LANG_ENGLISH} "Version $R0 is installed now. Your settings stay as they are."
LangString aepdUpdateSub ${LANG_RUSSIAN} "Сейчас установлена версия $R0. Настройки сохранятся."
LangString aepdUpdate ${LANG_ENGLISH} "Update"
LangString aepdUpdate ${LANG_RUSSIAN} "Обновить"

; Welcome: this version is installed
LangString aepdSameTitle ${LANG_ENGLISH} "Already [installed]"
LangString aepdSameTitle ${LANG_RUSSIAN} "Уже [установлено]"
LangString aepdSameSub ${LANG_ENGLISH} "Version ${VERSION} is on this computer. Install it again if something is broken."
LangString aepdSameSub ${LANG_RUSSIAN} "Это версия ${VERSION}. Если что-то не работает, установите её заново."
LangString aepdReinstall ${LANG_ENGLISH} "Reinstall"
LangString aepdReinstall ${LANG_RUSSIAN} "Переустановить"

; Welcome: a newer version is installed ($R0 is its version)
LangString aepdDowngradeTitle ${LANG_ENGLISH} "Go back to [${VERSION}]"
LangString aepdDowngradeTitle ${LANG_RUSSIAN} "Вернуться к [${VERSION}]"
LangString aepdDowngradeSub ${LANG_ENGLISH} "Version $R0 is installed now. This replaces it with ${VERSION}."
LangString aepdDowngradeSub ${LANG_RUSSIAN} "Сейчас установлена более новая версия $R0. Её заменит ${VERSION}."
LangString aepdReplace ${LANG_ENGLISH} "Replace"
LangString aepdReplace ${LANG_RUSSIAN} "Заменить"

; Progress
LangString aepdInstallingTitle ${LANG_ENGLISH} "Installing"
LangString aepdInstallingTitle ${LANG_RUSSIAN} "Установка"
LangString aepdInstallingSub ${LANG_ENGLISH} "AEP Downgrader ${VERSION} will be ready in a few seconds."
LangString aepdInstallingSub ${LANG_RUSSIAN} "AEP Downgrader ${VERSION} будет готов через пару секунд."
LangString aepdUpdatingTitle ${LANG_ENGLISH} "Updating to [${VERSION}]"
LangString aepdUpdatingTitle ${LANG_RUSSIAN} "Обновление до [${VERSION}]"
LangString aepdUpdatingRestartSub ${LANG_ENGLISH} "The app opens again by itself when it is done."
LangString aepdUpdatingRestartSub ${LANG_RUSSIAN} "Когда всё будет готово, приложение откроется само."
LangString aepdUpdatingSub ${LANG_ENGLISH} "Your settings stay as they are. This takes a few seconds."
LangString aepdUpdatingSub ${LANG_RUSSIAN} "Настройки сохранятся. Это займёт пару секунд."
LangString aepdStatusCopy ${LANG_ENGLISH} "Copying files…"
LangString aepdStatusCopy ${LANG_RUSSIAN} "Копируем файлы…"
LangString aepdStatusReplace ${LANG_ENGLISH} "Replacing the app…"
LangString aepdStatusReplace ${LANG_RUSSIAN} "Заменяем приложение…"
LangString aepdStatusWebview ${LANG_ENGLISH} "Downloading WebView2 from Microsoft…"
LangString aepdStatusWebview ${LANG_RUSSIAN} "Скачиваем WebView2 у Microsoft…"
LangString aepdStatusShortcuts ${LANG_ENGLISH} "Adding it to the Start menu…"
LangString aepdStatusShortcuts ${LANG_RUSSIAN} "Добавляем в меню «Пуск»…"

; Finish
LangString aepdDoneTitle ${LANG_ENGLISH} "All [set]"
LangString aepdDoneTitle ${LANG_RUSSIAN} "Всё [готово]"
LangString aepdDoneSub ${LANG_ENGLISH} "Find AEP Downgrader in the Start menu."
LangString aepdDoneSub ${LANG_RUSSIAN} "AEP Downgrader теперь есть в меню «Пуск»."
LangString aepdOpen ${LANG_ENGLISH} "Open AEP Downgrader"
LangString aepdOpen ${LANG_RUSSIAN} "Открыть AEP Downgrader"
LangString aepdClose ${LANG_ENGLISH} "Close"
LangString aepdClose ${LANG_RUSSIAN} "Закрыть"
LangString aepdDesktop ${LANG_ENGLISH} "Add a desktop shortcut"
LangString aepdDesktop ${LANG_RUSSIAN} "Ярлык на рабочем столе"

; Something failed
LangString aepdErrorTitle ${LANG_ENGLISH} "Something went [wrong]"
LangString aepdErrorTitle ${LANG_RUSSIAN} "Что-то пошло [не так]"
LangString aepdErrorSub ${LANG_ENGLISH} "The installation was interrupted. Close this window and run the installer again."
LangString aepdErrorSub ${LANG_RUSSIAN} "Установка прервалась. Закройте окно и запустите установщик ещё раз."
LangString aepdUnErrorSub ${LANG_ENGLISH} "The removal was interrupted. Close this window and try again."
LangString aepdUnErrorSub ${LANG_RUSSIAN} "Удаление прервалось. Закройте окно и попробуйте ещё раз."

; Uninstaller
LangString aepdUninstallTitle ${LANG_ENGLISH} "Remove [AEP Downgrader]?"
LangString aepdUninstallTitle ${LANG_RUSSIAN} "Удалить [AEP Downgrader]?"
LangString aepdUninstallSub ${LANG_ENGLISH} "Your After Effects files stay where they are."
LangString aepdUninstallSub ${LANG_RUSSIAN} "Файлы After Effects останутся на месте."
LangString aepdRemove ${LANG_ENGLISH} "Remove"
LangString aepdRemove ${LANG_RUSSIAN} "Удалить"
LangString aepdKeep ${LANG_ENGLISH} "Keep it"
LangString aepdKeep ${LANG_RUSSIAN} "Оставить"
LangString aepdDeleteData ${LANG_ENGLISH} "Also delete settings"
LangString aepdDeleteData ${LANG_RUSSIAN} "Удалить и настройки"
LangString aepdRemovingTitle ${LANG_ENGLISH} "Removing"
LangString aepdRemovingTitle ${LANG_RUSSIAN} "Удаление"
LangString aepdRemovingSub ${LANG_ENGLISH} "This takes a few seconds."
LangString aepdRemovingSub ${LANG_RUSSIAN} "Это займёт пару секунд."
LangString aepdStatusRemove ${LANG_ENGLISH} "Removing files…"
LangString aepdStatusRemove ${LANG_RUSSIAN} "Удаляем файлы…"
LangString aepdRemovedTitle ${LANG_ENGLISH} "All [done]"
LangString aepdRemovedTitle ${LANG_RUSSIAN} "[Готово]"
LangString aepdRemovedSub ${LANG_ENGLISH} "AEP Downgrader is removed. Thanks for giving it a try."
LangString aepdRemovedSub ${LANG_RUSSIAN} "AEP Downgrader удалён. Спасибо, что попробовали."
