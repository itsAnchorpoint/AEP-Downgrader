import { beforeEach, describe, expect, it } from "vitest";
import { initialState, useAppStore } from "../store";
import { THEMES, usesOwnAccent } from "./settings";

describe("themes", () => {
  it("offers system, light, dark and After Effects", () => {
    expect([...THEMES]).toEqual(["system", "light", "dark", "ae"]);
  });

  it("only the After Effects theme brings its own accent", () => {
    expect(THEMES.filter(usesOwnAccent)).toEqual(["ae"]);
  });

  describe("in the store", () => {
    beforeEach(() => useAppStore.setState({ ...initialState }));

    it("keeps the chosen accent while the After Effects theme is active", () => {
      useAppStore.getState().updateSettings({ accent: "aqua" });
      useAppStore.getState().updateSettings({ theme: "ae" });
      useAppStore.getState().updateSettings({ theme: "dark" });
      expect(useAppStore.getState().settings).toMatchObject({ theme: "dark", accent: "aqua" });
    });
  });
});
