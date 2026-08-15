// Command-line descriptive statistics: pass numbers as arguments, get count,
// mean, standard deviation, median and outliers. With no arguments it falls
// back to a built-in sample so the script is self-demonstrating.
//
//   node argv_stats.js 12 15 11 42 --precision 3

const DEFAULT_SAMPLE = [12, 15, 11, 42, 19, 13, 14, 16, 91, 15];

function parseArgs(argv) {
  const numbers = [];
  const options = { precision: 2, verbose: false };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === "--verbose") {
      options.verbose = true;
      continue;
    }
    if (arg === "--precision") {
      options.precision = +argv[i + 1];
      i += 1;
      continue;
    }
    const value = +arg;
    if (value !== value) {
      console.warn("ignoring non-numeric argument: " + arg);
      continue;
    }
    numbers.push(value);
  }
  return { numbers: numbers, options: options };
}

function mean(values) {
  let sum = 0;
  for (const value of values) {
    sum += value;
  }
  return sum / values.length;
}

function standardDeviation(values) {
  const average = mean(values);
  let sumSquares = 0;
  for (const value of values) {
    sumSquares += (value - average) * (value - average);
  }
  return Math.sqrt(sumSquares / values.length);
}

function median(values) {
  const sorted = values.slice().sort((left, right) => left - right);
  const middle = Math.floor(sorted.length / 2);
  if (sorted.length % 2 === 1) return sorted[middle];
  return (sorted[middle - 1] + sorted[middle]) / 2;
}

function outliers(values, sigma = 2) {
  const average = mean(values);
  const spread = standardDeviation(values);
  return values.filter((value) => Math.abs(value - average) > sigma * spread);
}

const parsed = parseArgs(process.argv.slice(2));
const numbers = parsed.numbers.length > 0 ? parsed.numbers : DEFAULT_SAMPLE;
const precision = parsed.options.precision;

if (parsed.numbers.length === 0) {
  console.log("no numbers given; using the built-in sample");
}

console.log("n       ", numbers.length);
console.log("min     ", Math.min.apply(null, numbers));
console.log("max     ", Math.max.apply(null, numbers));
console.log("mean    ", mean(numbers).toFixed(precision));
console.log("median  ", median(numbers).toFixed(precision));
console.log("stddev  ", standardDeviation(numbers).toFixed(precision));

const far = outliers(numbers);
console.log("outliers", far.length === 0 ? "none" : far.join(" "));

if (parsed.options.verbose) {
  console.log("sorted:", numbers.slice().sort((left, right) => left - right).join(" "));
}
