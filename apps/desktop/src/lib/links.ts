export const REPO_URL = "https://github.com/itsAnchorpoint/AEP-Downgrader";
export const SITE_URL = "https://aepdowngrader.com";

/** The donation page on the site, in the app's language (Boosty and Tribute are listed there). */
export const supportUrl = (language: string) =>
  language.startsWith("ru") ? `${SITE_URL}/ru/support/` : `${SITE_URL}/support/`;

/** Release page of a version, for installs the app cannot update itself. */
export const releaseUrl = (version: string) =>
  version ? `${REPO_URL}/releases/tag/v${version}` : `${REPO_URL}/releases/latest`;
