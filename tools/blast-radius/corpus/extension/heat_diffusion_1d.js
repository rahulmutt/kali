// Explicit finite-difference solution of the 1-D heat equation on a rod with
// fixed end temperatures. Runs a fixed number of steps and prints the profile
// at a few checkpoints.

const CELLS = 41;
const ALPHA = 0.00023;
const DX = 0.01;
const DT = 0.02;
const STEPS = 400;

const temperature = [];
for (let i = 0; i < CELLS; i++) {
  temperature[i] = 20;
}
temperature[0] = 100;
temperature[CELLS - 1] = 0;

const scratch = new Array(CELLS);

function step(current, next, coefficient) {
  next[0] = current[0];
  next[current.length - 1] = current[current.length - 1];
  for (let i = 1; i < current.length - 1; i++) {
    next[i] = current[i] + coefficient * (current[i - 1] - 2 * current[i] + current[i + 1]);
  }
}

function copyInto(source, target) {
  for (let i = 0; i < source.length; i++) {
    target[i] = source[i];
  }
}

function profileLine(values, sampleEvery = 8) {
  let line = "";
  for (let i = 0; i < values.length; i += sampleEvery) {
    line += values[i].toFixed(1) + " ";
  }
  return line.trim();
}

function totalHeat(values) {
  let sum = 0;
  for (const value of values) {
    sum += value;
  }
  return sum * DX;
}

const coefficient = (ALPHA * DT) / (DX * DX);
if (coefficient > 0.5) {
  console.warn("explicit scheme is unstable at coefficient " + coefficient);
}

console.log("stability coefficient:", coefficient.toFixed(4));
console.log("step 0:", profileLine(temperature));

for (let s = 1; s <= STEPS; s++) {
  step(temperature, scratch, coefficient);
  copyInto(scratch, temperature);
  if (s % 100 === 0) {
    console.log("step " + s + ": " + profileLine(temperature));
  }
}

console.log("heat integral:", totalHeat(temperature).toFixed(4));
console.log("ends held:", temperature[0] === 100 && temperature[CELLS - 1] === 0);
