import { describe, expect, it } from "vitest";
import en from "./en.json";
import ru from "./ru.json";

type Tree = { [key: string]: string | Tree };

// Plural forms differ per language (en: one/other, ru: one/few/many/other).
const PLURAL = /_(zero|one|two|few|many|other)$/;

function keys(tree: Tree, prefix = ""): string[] {
  return Object.entries(tree).flatMap(([k, v]) =>
    typeof v === "string" ? [(prefix + k).replace(PLURAL, "")] : keys(v, `${prefix}${k}.`),
  );
}

const unique = (list: string[]) => [...new Set(list)].sort();
function placeholdersByKey(tree: Tree): Record<string, string> {
  const result: Record<string, Set<string>> = {};
  for (const [key, text] of Object.entries(flat(tree))) {
    const base = key.replace(PLURAL, "");
    result[base] ??= new Set();
    (text.match(/\{\{\w+\}\}|\{\w+\}/g) ?? []).forEach((p) => result[base].add(p));
  }
  return Object.fromEntries(Object.entries(result).map(([k, v]) => [k, [...v].sort().join()]));
}

function flat(tree: Tree, prefix = ""): Record<string, string> {
  return Object.fromEntries(
    Object.entries(tree).flatMap(([k, v]) =>
      typeof v === "string" ? [[prefix + k, v]] : Object.entries(flat(v, `${prefix}${k}.`)),
    ),
  );
}

describe("catalogs", () => {
  it("en and ru define the same keys", () => {
    expect(unique(keys(ru))).toEqual(unique(keys(en)));
  });

  it("each key uses the same placeholders in both languages", () => {
    expect(placeholdersByKey(ru)).toEqual(placeholdersByKey(en));
  });

  it("ru has the extra plural forms Russian needs", () => {
    expect(ru.queue).toHaveProperty("convert_few");
    expect(ru.queue).toHaveProperty("convert_many");
  });
});
