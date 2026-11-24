// Usage: node .github/scripts/check-compat-table.mjs   (from the repository root; needs cargo)
//
// The compatibility table lives in crates/aep-core/src/compat.rs. The docs copy it, so this check
// fails (exit 1) when a copy differs from what `aep compat` prints:
// - README.md and docs/COMPATIBILITY.md: the whole table, row for row;
// - README.ru.md: the short "Версия | Статус" table, the same targets with the same statuses.
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

const problems = [];

const expected = execFileSync("cargo", ["run", "--quiet", "-p", "aep-cli", "--", "compat"], { encoding: "utf8" })
  .trimEnd()
  .split("\n");

// The table that starts with `header`: that line and every line after it that starts with "|".
function tableAt(file, header) {
  const lines = readFileSync(file, "utf8").split("\n");
  const start = lines.findIndex((line) => line.trim() === header);
  if (start === -1) {
    problems.push(`${file}: no table starting with "${header}"`);
    return null;
  }
  const rows = [];
  for (let i = start; i < lines.length && lines[i].trim().startsWith("|"); i++) rows.push(lines[i].trim());
  return rows;
}

for (const file of ["README.md", "docs/COMPATIBILITY.md"]) {
  const rows = tableAt(file, expected[0]);
  if (rows && rows.join("\n") !== expected.join("\n")) {
    problems.push(`${file}: the table differs from \`aep compat\`. Paste this:\n${expected.join("\n")}`);
  }
}

const statusRu = { Verified: "Проверено", Experimental: "Экспериментально" };
const expectedRu = [
  "| Версия | Статус |",
  "|---|---|",
  ...expected.slice(2).map((row) => {
    const [target, status] = row.split("|").slice(1, 3).map((cell) => cell.trim());
    return `| ${target.replace(/ \(\d+\.x\)$/, "")} | ${statusRu[status] ?? status} |`;
  }),
];
const rowsRu = tableAt("README.ru.md", expectedRu[0]);
if (rowsRu && rowsRu.join("\n") !== expectedRu.join("\n")) {
  problems.push(`README.ru.md: the table differs from \`aep compat\`. Paste this:\n${expectedRu.join("\n")}`);
}

if (problems.length) {
  for (const problem of problems) console.error(`check-compat-table: ${problem}\n`);
  process.exit(1);
}
console.log("check-compat-table: README.md, README.ru.md and docs/COMPATIBILITY.md match `aep compat`");
