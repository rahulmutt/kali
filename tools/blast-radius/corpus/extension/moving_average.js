// Smooth a noisy sensor series with a centred moving average and an
// exponentially weighted average, then report where the two disagree most.
// The samples come from a seeded generator so the run is reproducible.

let seed = 20260815;

function nextRandom() {
  seed = (seed * 1103515245 + 12345) % 2147483648;
  return seed / 2147483648;
}

function generateSeries(count) {
  const series = [];
  for (let i = 0; i < count; i++) {
    const trend = 20 + 6 * Math.sin(i / 9);
    const noise = (nextRandom() - 0.5) * 3;
    series.push(trend + noise);
  }
  return series;
}

function movingAverage(series, window = 5) {
  const half = Math.floor(window / 2);
  const out = [];
  for (let i = 0; i < series.length; i++) {
    let sum = 0;
    let used = 0;
    for (let k = i - half; k <= i + half; k++) {
      if (k < 0 || k >= series.length) continue;
      sum += series[k];
      used += 1;
    }
    out.push(sum / used);
  }
  return out;
}

function exponentialAverage(series, alpha = 0.25) {
  const out = [];
  let current = series[0];
  for (const value of series) {
    current = alpha * value + (1 - alpha) * current;
    out.push(current);
  }
  return out;
}

function largestGap(left, right) {
  let index = 0;
  let gap = 0;
  for (let i = 0; i < left.length; i++) {
    const delta = Math.abs(left[i] - right[i]);
    if (delta > gap) {
      gap = delta;
      index = i;
    }
  }
  return { index: index, gap: gap };
}

const series = generateSeries(60);
const smoothed = movingAverage(series, 7);
const exponential = exponentialAverage(series);

console.log("samples:", series.length);
console.log("raw spread:", (Math.max.apply(null, series) - Math.min.apply(null, series)).toFixed(3));
console.log("smoothed spread:", (Math.max.apply(null, smoothed) - Math.min.apply(null, smoothed)).toFixed(3));

const worst = largestGap(smoothed, exponential);
console.log("largest disagreement at sample", worst.index, "of", worst.gap.toFixed(3));

for (let i = 0; i < series.length; i += 12) {
  console.log(
    String(i).padStart(3) +
      series[i].toFixed(2).padStart(9) +
      smoothed[i].toFixed(2).padStart(9) +
      exponential[i].toFixed(2).padStart(9),
  );
}
