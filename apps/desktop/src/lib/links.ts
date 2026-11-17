export const REPO_URL = "https://github.com/itsAnchorpoint/AEP-Downgrader";

/** Release page of a version, for installs the app cannot update itself. */
export const releaseUrl = (version: string) =>
  version ? `${REPO_URL}/releases/tag/v${version}` : `${REPO_URL}/releases/latest`;
