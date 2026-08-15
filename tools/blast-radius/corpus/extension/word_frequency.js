// Rank the most frequent content words in a block of prose, the way a quick
// readability or keyword check does.

const TEXT =
  "The compiler reads the source, the source becomes a tree, and the tree " +
  "becomes code. A compiler that reads a tree it did not build is a compiler " +
  "that trusts a stranger. Trust the tree, but measure the tree first.";

const STOP_WORDS = ["the", "a", "an", "and", "of", "to", "in", "is", "it", "that", "but"];

function tokenize(text) {
  const words = [];
  let current = "";
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    const isWordChar =
      (ch >= "a" && ch <= "z") || (ch >= "A" && ch <= "Z") || ch === "'";
    if (isWordChar) {
      current += ch;
    } else if (current.length > 0) {
      words.push(current.toLowerCase());
      current = "";
    }
  }
  if (current.length > 0) {
    words.push(current.toLowerCase());
  }
  return words;
}

function isStopWord(word) {
  return STOP_WORDS.indexOf(word) >= 0;
}

function countWords(words) {
  const counts = {};
  for (const word of words) {
    if (isStopWord(word)) continue;
    if (counts[word] === undefined) {
      counts[word] = 0;
    }
    counts[word] += 1;
  }
  return counts;
}

function ranked(counts, limit = 5) {
  const entries = [];
  for (const word of Object.keys(counts)) {
    entries.push({ word: word, count: counts[word] });
  }
  entries.sort(function (left, right) {
    if (right.count !== left.count) return right.count - left.count;
    return left.word < right.word ? -1 : 1;
  });
  return entries.slice(0, limit);
}

const words = tokenize(TEXT);
console.log("tokens:", words.length);
console.log("distinct content words:", Object.keys(countWords(words)).length);

for (const entry of ranked(countWords(words))) {
  console.log(entry.word + " " + entry.count);
}
