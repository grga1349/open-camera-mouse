// Header rows of the Markdown tables in a manual page, so a test can check that every column
// has a name a screen reader can announce (axe `empty-table-header`).

const separator = /^\|?\s*:?-+:?\s*(\|\s*:?-+:?\s*)*\|?\s*$/;

/** Cells of a table row: `| a | b |` → `["a", "b"]` (outer pipes optional). */
export function cells(row: string): string[] {
  let s = row.trim();
  if (s.startsWith("|")) s = s.slice(1);
  if (s.endsWith("|")) s = s.slice(0, -1);
  return s.split("|").map((c) => c.trim());
}

/** Each table's header row (the line above a `|---|` separator), outside fenced code. */
export function tableHeaders(md: string): { line: number; cells: string[] }[] {
  const lines = md.split("\n");
  const headers: { line: number; cells: string[] }[] = [];
  let fenced = false;
  for (let i = 0; i < lines.length; i++) {
    if (/^\s*(```|~~~)/.test(lines[i])) fenced = !fenced;
    if (fenced || i === 0) continue;
    const above = lines[i - 1];
    if (separator.test(lines[i]) && above.includes("|")) {
      headers.push({ line: i, cells: cells(above) });
    }
  }
  return headers;
}
