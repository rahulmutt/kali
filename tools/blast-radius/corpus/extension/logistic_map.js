// Iterate the logistic map at several growth rates, discard the transient, and
// bin the attractor -- the cheapest way to see a period-doubling cascade
// without plotting anything.

const TRANSIENT = 500;
const SAMPLES = 2000;
const BINS = 40;

function orbit(rate, seed, samples) {
  let x = seed;
  for (let i = 0; i < TRANSIENT; i++) {
    x = rate * x * (1 - x);
  }
  const values = [];
  for (let i = 0; i < samples; i++) {
    x = rate * x * (1 - x);
    values.push(x);
  }
  return values;
}

function histogram(values, bins) {
  const counts = new Array(bins).fill(0);
  for (const value of values) {
    let index = Math.floor(value * bins);
    if (index >= bins) index = bins - 1;
    if (index < 0) index = 0;
    counts[index] += 1;
  }
  return counts;
}

function occupiedBins(counts) {
  return counts.filter((count) => count > 0).length;
}

function sparkline(counts) {
  const peak = Math.max.apply(null, counts);
  const glyphs = " .:-=+*#";
  let line = "";
  for (const count of counts) {
    const level = peak === 0 ? 0 : Math.round((count / peak) * (glyphs.length - 1));
    line += glyphs.charAt(level);
  }
  return line;
}

const RATES = [2.8, 3.2, 3.5, 3.56, 3.83, 3.99];

for (const rate of RATES) {
  const values = orbit(rate, 0.4, SAMPLES);
  const counts = histogram(values, BINS);
  const occupied = occupiedBins(counts);
  const label = "r=" + rate.toFixed(2);
  console.log(label.padEnd(8) + "bins=" + String(occupied).padStart(3) + "  " + sparkline(counts));
}

const chaotic = orbit(3.99, 0.4, SAMPLES);
let mean = 0;
for (const value of chaotic) {
  mean += value / chaotic.length;
}
console.log("mean of chaotic orbit:", mean.toFixed(4));
console.log("orbit stays in the unit interval:", Math.min.apply(null, chaotic) >= 0);
