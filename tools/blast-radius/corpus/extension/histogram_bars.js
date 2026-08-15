// Draw an ASCII histogram of latency samples: fixed buckets, a scaled bar per
// bucket, and the percentiles underneath. The chart you print when a real
// plotting stack is not worth the dependency.

const SAMPLES = [
  12, 15, 11, 19, 22, 14, 13, 41, 17, 16, 18, 12, 13, 15, 27, 33, 21, 14,
  16, 19, 11, 12, 58, 24, 23, 18, 17, 15, 14, 13, 12, 11, 19, 26, 31, 44,
  16, 17, 18, 20, 22, 25, 29, 35, 39, 47, 52, 61, 74, 96,
];

function bucketise(values, bucketWidth = 10) {
  const buckets = [];
  for (const value of values) {
    const index = Math.floor(value / bucketWidth);
    while (buckets.length <= index) {
      buckets.push(0);
    }
    buckets[index] += 1;
  }
  return buckets;
}

function percentile(values, fraction) {
  const sorted = values.slice().sort(function (left, right) {
    return left - right;
  });
  const position = Math.min(sorted.length - 1, Math.floor(fraction * sorted.length));
  return sorted[position];
}

function bar(count, peak, width = 40) {
  const filled = peak === 0 ? 0 : Math.round((count / peak) * width);
  return "#".repeat(filled) + "-".repeat(width - filled);
}

function label(index, bucketWidth) {
  const low = index * bucketWidth;
  return (String(low) + "-" + String(low + bucketWidth - 1)).padStart(8);
}

const BUCKET_WIDTH = 10;
const buckets = bucketise(SAMPLES, BUCKET_WIDTH);
let peak = 0;
for (const count of buckets) {
  if (count > peak) peak = count;
}

console.log("samples:", SAMPLES.length, "buckets:", buckets.length);
for (let i = 0; i < buckets.length; i++) {
  console.log(label(i, BUCKET_WIDTH) + " ms | " + bar(buckets[i], peak) + " " + buckets[i]);
}

console.log("p50 " + percentile(SAMPLES, 0.5) + " ms");
console.log("p90 " + percentile(SAMPLES, 0.9) + " ms");
console.log("p99 " + percentile(SAMPLES, 0.99) + " ms");
console.log("=".repeat(60) + " end of report");
