import { describe, expect, it } from "vitest";
import { DEFAULT_PATTERN, outputName } from "./pattern";

describe("outputName", () => {
  it("fills {name} and {year} and keeps the extension", () => {
    expect(outputName("{name}_AE{year}", "Intro_v05.aep", 2024)).toBe("Intro_v05_AE2024.aep");
  });

  it("keeps only the last extension and dots inside the name", () => {
    expect(outputName("{name}-{year}", "my.comp.v2.aet", "2023")).toBe("my.comp.v2-2023.aet");
  });

  it("falls back to the default pattern when empty", () => {
    expect(outputName("   ", "A.ffx", 2025)).toBe(outputName(DEFAULT_PATTERN, "A.ffx", 2025));
  });

  it("replaces characters that are illegal in file names", () => {
    expect(outputName("{name}:{year}/x", "A.aep", 2024)).toBe("A_2024_x.aep");
  });

  it("handles files without an extension", () => {
    expect(outputName("{name}_{year}", "README", 2024)).toBe("README_2024");
  });
});
