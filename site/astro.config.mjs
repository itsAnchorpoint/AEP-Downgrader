// @ts-check
import { defineConfig } from "astro/config";
import sitemap from "@astrojs/sitemap";

export default defineConfig({
  site: "https://aepdowngrader.com",
  trailingSlash: "always",
  build: { format: "directory", inlineStylesheets: "always" },
  integrations: [
    sitemap({
      i18n: { defaultLocale: "en", locales: { en: "en", ru: "ru" } },
      filter: (page) => !page.includes("/404"),
    }),
  ],
});
