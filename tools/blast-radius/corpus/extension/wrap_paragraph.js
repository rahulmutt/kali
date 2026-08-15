// Hard-wrap prose to a column width, honouring an optional indent. Every
// project that emits help text or commit messages ends up with one of these.

const PARAGRAPH =
  "A compiler that silently miscompiles is worse than one that refuses, " +
  "because the refusal is a message and the miscompilation is a lie. The whole " +
  "point of measuring blast radius is to decide which lie to stop telling first.";

function wrap(text, width = 72, indent = "") {
  const words = text.split(" ").filter((word) => word.length > 0);
  const lines = [];
  let line = indent;
  let started = false;
  for (const word of words) {
    const candidate = started ? line + " " + word : line + word;
    if (started && candidate.length > width) {
      lines.push(line);
      line = indent + word;
    } else {
      line = candidate;
      started = true;
    }
  }
  if (started) lines.push(line);
  return lines;
}

const wrapped = wrap(PARAGRAPH, 60);
console.log("wrapped into", wrapped.length, "lines");
wrapped.forEach(function (line) {
  console.log(line);
});

console.log("--- quoted at width 50 ---");
for (const line of wrap(PARAGRAPH, 50, "> ")) {
  console.log(line);
}

const longest = wrap(PARAGRAPH, 60).reduce(function (best, line) {
  return line.length > best ? line.length : best;
}, 0);
console.log("longest line:", longest);
