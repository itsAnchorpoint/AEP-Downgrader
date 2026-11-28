import { describe, expect, it } from "vitest";
import { supportUrl } from "./links";

describe("supportUrl", () => {
  it("opens the support page in the app's language", () => {
    expect(supportUrl("ru")).toBe("https://aepdowngrader.com/ru/support/");
    expect(supportUrl("ru-RU")).toBe("https://aepdowngrader.com/ru/support/");
    expect(supportUrl("en")).toBe("https://aepdowngrader.com/support/");
  });
});
