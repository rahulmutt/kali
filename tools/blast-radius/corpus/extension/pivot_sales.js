// Pivot a long sales table into a quarter-by-region matrix with margins, and
// render it as fixed-width text. This is the report people ask for after they
// have seen the flat export once.

const SALES = [
  { region: "north", quarter: "Q1", amount: 1200 },
  { region: "north", quarter: "Q2", amount: 1450 },
  { region: "north", quarter: "Q4", amount: 1800 },
  { region: "south", quarter: "Q1", amount: 980 },
  { region: "south", quarter: "Q3", amount: 1120 },
  { region: "east", quarter: "Q2", amount: 2100 },
  { region: "east", quarter: "Q3", amount: 1950 },
  { region: "east", quarter: "Q4", amount: 2400 },
  { region: "west", quarter: "Q1", amount: 640 },
  { region: "west", quarter: "Q4", amount: 720 },
];

const QUARTERS = ["Q1", "Q2", "Q3", "Q4"];

function distinct(records, field) {
  const seen = {};
  const values = [];
  for (const record of records) {
    if (seen[record[field]]) continue;
    seen[record[field]] = true;
    values.push(record[field]);
  }
  return values.sort();
}

function pivot(records, rowField, columnField, valueField) {
  const rowKeys = distinct(records, rowField);
  const grid = [];
  for (let r = 0; r < rowKeys.length; r++) {
    grid.push(new Array(QUARTERS.length).fill(0));
  }
  for (const record of records) {
    const r = rowKeys.indexOf(record[rowField]);
    const c = QUARTERS.indexOf(record[columnField]);
    if (r < 0 || c < 0) {
      console.warn("dropping record outside the pivot axes");
      continue;
    }
    grid[r][c] += record[valueField];
  }
  return { rowKeys: rowKeys, grid: grid };
}

function rowTotal(row) {
  let sum = 0;
  for (const cell of row) {
    sum += cell;
  }
  return sum;
}

function columnTotals(grid) {
  const totals = new Array(QUARTERS.length).fill(0);
  for (const row of grid) {
    for (let c = 0; c < row.length; c++) {
      totals[c] += row[c];
    }
  }
  return totals;
}

function renderRow(label, cells, width = 8) {
  let line = label.padEnd(10);
  for (const cell of cells) {
    line += String(cell).padStart(width);
  }
  return line;
}

const table = pivot(SALES, "region", "quarter", "amount");

console.log(renderRow("region", QUARTERS.concat(["total"])));
for (let r = 0; r < table.rowKeys.length; r++) {
  const row = table.grid[r];
  console.log(renderRow(table.rowKeys[r], row.concat([rowTotal(row)])));
}

const totals = columnTotals(table.grid);
console.log(renderRow("total", totals.concat([rowTotal(totals)])));
console.log("grand total agrees:", rowTotal(totals) === SALES.reduce((sum, s) => sum + s.amount, 0));
