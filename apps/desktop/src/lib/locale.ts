export type Language = "en" | "ru";

/** Russian for any ru-* OS locale, English otherwise. */
export function detectLanguage(locale: string = navigator.language): Language {
  return locale.toLowerCase().startsWith("ru") ? "ru" : "en";
}
