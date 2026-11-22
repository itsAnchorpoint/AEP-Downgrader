# Writing

How AEP Downgrader talks, so the app, the installers, the website and the docs sound like one product in both languages. The visual side is in [DESIGN.md](DESIGN.md).

## Voice

- Say what happens to the user's files, plainly. Short sentences, no marketing words.
- Promise only what was checked. A target that has not been opened in a real After Effects is Experimental, everywhere.
- When something can't be done, say why and what to do instead.
- Address the reader as "you" (Russian: «вы», lowercase).

## Terms

| English | Russian | Notes |
|---|---|---|
| project, template, preset | проект, шаблон, пресет | `.aep`, `.aet`, `.ffx`. "Files" when it means all three. |
| copy | копия | The file the app writes. |
| original | оригинал | The file the user dropped. Never modified. «Исходный файл» is fine in the name pattern hint. |
| convert, conversion | конвертировать, сконвертировать, конвертация | Not "downgrade" in the interface. |
| target, the version you convert to | версия | The picker label is "Convert to" / «Сохранить для». |
| After Effects 2024 | After Effects 2024 | Year names in the interface. Version numbers (24.x, 24.5) only in technical text. Never "AE" in interface text. |
| Verified, Experimental | Проверено, Экспериментально | See [COMPATIBILITY.md](COMPATIBILITY.md). |
| report | отчёт | What will change in a copy, per file. |
| activity log | журнал действий | |
| Settings, What's new, Update | Настройки, Что нового, Обновить | |
| Stop | Остановить | |
| track matte | слой-маска | |
| plugin | плагин | Not "plug-in". |
| Apple silicon, Intel | Apple silicon, Intel | |

Names of After Effects features and menus stay in English in both languages (**Save a Copy As**, Shadow Color): most users run After Effects in English.

## English

- American spelling: color, license, recognize, behavior.
- Interface text uses contractions (doesn't, can't, isn't) and the typographic apostrophe ’.
- Sentence case for buttons, labels and headings: "Show in folder", not "Show In Folder".
- No serial comma: "projects, templates and presets".

## Russian

- Quotes «ёлочки», inside them „лапки“. Button names go in «»: нажмите «Обновить».
- Always write ё.
- A colon or a comma instead of a dash where one works. Keep the dash where grammar needs it.
- No officialese: «проверьте доступ к папке», not «осуществите проверку доступа».

## Names the operating system shows

Write them exactly as the system shows them, in the user's language. These were checked on real systems in November 2026.

| Where | English | Russian |
|---|---|---|
| Windows SmartScreen | Windows protected your PC → More info → Run anyway | Система Windows защитила ваш компьютер → Подробнее → Выполнить в любом случае |
| macOS, first launch (15 and newer) | "Not Opened" dialog: press **Done**, not Move to Trash | Окно «не был открыт»: нажмите «Готово», а не «Переместить в Корзину» |
| macOS, then | System Settings → Privacy & Security → Open Anyway | Системные настройки → Конфиденциальность и безопасность → Все равно открыть |

Apple writes «Все равно» without ё; keep the system's spelling inside quotes.

## Where the words live

- App: `apps/desktop/src/i18n/en.json` and `ru.json` (keys must match: `src/i18n/parity.test.ts`). Release notes shown in the app: `apps/desktop/src/content/whatsnew.en.md` and `whatsnew.ru.md`.
- Windows installer: `apps/desktop/src-tauri/windows/hooks.nsh`.
- macOS disk image: `apps/desktop/src-tauri/dmg/background.html` (English only).
- Website: `site/`.
- Docs: `README.md`, `README.ru.md`, `CHANGELOG.md`, `docs/`.
