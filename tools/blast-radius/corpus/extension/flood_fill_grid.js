// Flood fill a character grid and report the connected regions: the paint
// bucket, and the same walk reused to count rooms in a map.

const MAP = [
  "##########",
  "#..#....##",
  "#..#..#..#",
  "#..####..#",
  "#........#",
  "####..####",
  "#..#..#..#",
  "#..#..#..#",
  "##########",
];

function toGrid(lines) {
  const grid = [];
  for (const line of lines) {
    grid.push(line.split(""));
  }
  return grid;
}

function inBounds(grid, row, col) {
  return row >= 0 && row < grid.length && col >= 0 && col < grid[row].length;
}

function fill(grid, startRow, startCol, paint) {
  const target = grid[startRow][startCol];
  if (target === paint) return 0;
  const stack = [{ row: startRow, col: startCol }];
  const steps = [
    { row: -1, col: 0 },
    { row: 1, col: 0 },
    { row: 0, col: -1 },
    { row: 0, col: 1 },
  ];
  let painted = 0;

  while (stack.length > 0) {
    const cell = stack.pop();
    if (!inBounds(grid, cell.row, cell.col)) continue;
    if (grid[cell.row][cell.col] !== target) continue;
    grid[cell.row][cell.col] = paint;
    painted += 1;
    for (const step of steps) {
      stack.push({ row: cell.row + step.row, col: cell.col + step.col });
    }
  }
  return painted;
}

function render(grid) {
  const lines = [];
  for (const row of grid) {
    lines.push(row.join(""));
  }
  return lines.join("\n");
}

function regions(lines) {
  const grid = toGrid(lines);
  const labels = "abcdefghijklmnopqrstuvwxyz";
  const sizes = [];
  let next = 0;
  for (let row = 0; row < grid.length; row++) {
    for (let col = 0; col < grid[row].length; col++) {
      if (grid[row][col] !== ".") continue;
      const size = fill(grid, row, col, labels[next]);
      sizes.push({ label: labels[next], size: size });
      next += 1;
    }
  }
  return { grid: grid, sizes: sizes };
}

const painted = toGrid(MAP);
const filled = fill(painted, 1, 1, "o");
console.log(render(painted));
console.log("cells painted from (1,1):", filled);

const found = regions(MAP);
console.log(render(found.grid));
console.log("regions:", found.sizes.length);
for (const region of found.sizes) {
  console.log("region " + region.label + ": " + region.size + " cells");
}

let open = 0;
for (const line of MAP) {
  open += line.split("").filter((ch) => ch === ".").length;
}
console.log("every open cell is accounted for:", open === found.sizes.reduce((sum, r) => sum + r.size, 0));
