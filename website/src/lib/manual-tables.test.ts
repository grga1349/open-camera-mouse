import { describe, expect, test } from "bun:test";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { cells, tableHeaders } from "./manual-tables";

describe("tableHeaders", () => {
  test("finds the header row above the separator", () => {
    const md = "Text\n\n| | Normal | Larger |\n|---|:--|--:|\n| Text | 14 | 16 |\n";
    expect(tableHeaders(md)).toEqual([{ line: 3, cells: ["", "Normal", "Larger"] }]);
  });

  test("ignores tables inside fenced code", () => {
    expect(tableHeaders("```\n| a |\n|---|\n```\n")).toEqual([]);
  });

  test("outer pipes are optional", () => {
    expect(cells("a | b")).toEqual(["a", "b"]);
  });
});

describe("manual tables", () => {
  const dir = join(import.meta.dir, "../content/manual");
  for (const file of readdirSync(dir).filter((f) => f.endsWith(".md"))) {
    test(`${file}: every table column has a header`, () => {
      const md = readFileSync(join(dir, file), "utf8");
      const empty = tableHeaders(md)
        .filter((h) => h.cells.some((c) => c === ""))
        .map((h) => `line ${h.line}: ${h.cells.join(" | ")}`);
      expect(empty).toEqual([]);
    });
  }
});
