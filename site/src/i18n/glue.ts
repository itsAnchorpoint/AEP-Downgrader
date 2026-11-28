/**
 * Keeps "After Effects" (and "After Effects 2024", "After Effects {year}") on one line: every string of a
 * language module goes through this, so the copy itself can use plain spaces.
 */
const NAME = /After Effects(?: (?=\d{4}\b|\{year\}))?/g;

export function glue<T>(value: T): T {
  if (typeof value === "string") return value.replace(NAME, (m) => m.replace(/ /g, " ")) as T;
  if (Array.isArray(value)) return value.map(glue) as T;
  if (value && typeof value === "object")
    return Object.fromEntries(Object.entries(value).map(([k, v]) => [k, glue(v)])) as T;
  return value;
}
