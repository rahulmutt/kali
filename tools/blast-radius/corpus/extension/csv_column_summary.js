// Summarise a small CSV export: row count, and per-column statistics for the
// columns that hold numbers. Malformed rows are reported and skipped rather
// than aborting the run, because export files routinely contain a few.

const CSV = [
  "region,units,revenue",
  "north,120,4380.5",
  "south,98,3512.25",
  "east,143,5210",
  "west,,1980.75",
  "north,77,2410.5",
  "truncated,41",
].join("\n");

function parseCsv(text) {
  const lines = text.split("\n");
  const header = lines[0].split(",");
  const rows = [];
  for (let i = 1; i < lines.length; i++) {
    const line = lines[i];
    if (line.length === 0) continue;
    const cells = line.split(",");
    if (cells.length !== header.length) {
      console.warn("skipping malformed row " + i + ": " + line);
      continue;
    }
    const row = {};
    for (let c = 0; c < header.length; c++) {
      row[header[c]] = cells[c];
    }
    rows.push(row);
  }
  return { header: header, rows: rows };
}

function looksNumeric(cell) {
  if (cell.length === 0) return false;
  const value = +cell;
  return value === value;
}

function columnStats(rows, column) {
  let count = 0;
  let sum = 0;
  let min = Infinity;
  let max = -Infinity;
  for (const row of rows) {
    const cell = row[column];
    if (!looksNumeric(cell)) continue;
    const value = +cell;
    count += 1;
    sum += value;
    if (value < min) min = value;
    if (value > max) max = value;
  }
  const mean = count === 0 ? 0 : sum / count;
  return { count: count, sum: sum, min: min, max: max, mean: mean };
}

function isNumericColumn(rows, column) {
  let seen = 0;
  for (const row of rows) {
    if (row[column].length === 0) continue;
    if (!looksNumeric(row[column])) return false;
    seen += 1;
  }
  return seen > 0;
}

const table = parseCsv(CSV);
console.log("columns:", table.header.join(" | "));
console.log("rows:", table.rows.length);

for (const column of table.header) {
  if (!isNumericColumn(table.rows, column)) continue;
  const stats = columnStats(table.rows, column);
  console.log(
    column + ": n=" + stats.count + " sum=" + stats.sum + " min=" + stats.min +
      " max=" + stats.max + " mean=" + stats.mean.toFixed(2),
  );
}
