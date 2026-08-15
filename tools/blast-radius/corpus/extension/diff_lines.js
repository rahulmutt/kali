// Line diff between two revisions of a file: longest common subsequence, then
// a unified-style hunk listing. What a review tool prints before anyone has
// wired up a real diff library.

const OLD = [
  "fn main() {",
  "    let config = load();",
  "    let input = read(config.path);",
  "    let ast = parse(input);",
  "    emit(ast);",
  "}",
].join("\n");

const NEW = [
  "fn main() {",
  "    let config = load();",
  "    let input = read(config.path);",
  "    let ast = parse(input);",
  "    let checked = check(ast);",
  "    emit(checked);",
  "    log(\"done\");",
  "}",
].join("\n");

function lcsTable(left, right) {
  const table = [];
  for (let i = 0; i <= left.length; i++) {
    table.push(new Array(right.length + 1).fill(0));
  }
  for (let i = 1; i <= left.length; i++) {
    for (let j = 1; j <= right.length; j++) {
      if (left[i - 1] === right[j - 1]) {
        table[i][j] = table[i - 1][j - 1] + 1;
      } else {
        table[i][j] = Math.max(table[i - 1][j], table[i][j - 1]);
      }
    }
  }
  return table;
}

function diff(left, right) {
  const table = lcsTable(left, right);
  const out = [];
  let i = left.length;
  let j = right.length;
  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && left[i - 1] === right[j - 1]) {
      out.push({ sign: " ", text: left[i - 1] });
      i -= 1;
      j -= 1;
    } else if (j > 0 && (i === 0 || table[i][j - 1] >= table[i - 1][j])) {
      out.push({ sign: "+", text: right[j - 1] });
      j -= 1;
    } else {
      out.push({ sign: "-", text: left[i - 1] });
      i -= 1;
    }
  }
  return out.reverse();
}

function summarise(changes) {
  let added = 0;
  let removed = 0;
  for (const change of changes) {
    if (change.sign === "+") added += 1;
    if (change.sign === "-") removed += 1;
  }
  return { added: added, removed: removed, unchanged: changes.length - added - removed };
}

const oldLines = OLD.split("\n");
const newLines = NEW.split("\n");

let changes = [];
changes = diff(oldLines, newLines);

for (const change of changes) {
  console.log(change.sign + change.text);
}

const counts = summarise(changes);
console.log("+" + counts.added + " -" + counts.removed + " =" + counts.unchanged);
console.log("identical files produce no edits:", summarise(diff(oldLines, oldLines)).added === 0);
console.log("common prefix length:", lcsTable(oldLines, newLines)[oldLines.length][newLines.length]);
