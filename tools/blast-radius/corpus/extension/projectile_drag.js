// Integrate a projectile with quadratic air drag using RK-free explicit Euler,
// and report range, apex and flight time. Small ballistics tables get built
// exactly this way.

const GRAVITY = 9.80665;
const AIR_DENSITY = 1.225;
const DRAG_COEFFICIENT = 0.47;
const RADIUS = 0.037;
const MASS = 0.145;
const DT = 0.001;

const area = Math.PI * RADIUS * RADIUS;
const dragFactor = (0.5 * AIR_DENSITY * DRAG_COEFFICIENT * area) / MASS;

function simulate(speed, degrees) {
  const radians = (degrees * Math.PI) / 180;
  let x = 0;
  let y = 0;
  let vx = speed * Math.cos(radians);
  let vy = speed * Math.sin(radians);
  let time = 0;
  let apex = 0;

  while (y >= 0) {
    const v = Math.sqrt(vx * vx + vy * vy);
    const ax = -dragFactor * v * vx;
    const ay = -GRAVITY - dragFactor * v * vy;
    vx += ax * DT;
    vy += ay * DT;
    x += vx * DT;
    y += vy * DT;
    time += DT;
    if (y > apex) apex = y;
    if (time > 60) break;
  }

  return { range: x, apex: apex, time: time, degrees: degrees, speed: speed };
}

function best(results) {
  let winner = results[0];
  for (const result of results) {
    if (result.range > winner.range) winner = result;
  }
  return winner;
}

const results = [];
for (let degrees = 20; degrees <= 60; degrees += 5) {
  results.push(simulate(40, degrees));
}

console.log("angle  range(m)  apex(m)  flight(s)");
results.forEach(function (result) {
  console.log(
    String(result.degrees).padStart(5) +
      result.range.toFixed(2).padStart(10) +
      result.apex.toFixed(2).padStart(9) +
      result.time.toFixed(2).padStart(11),
  );
});

const optimal = best(results);
console.log("best angle:", optimal.degrees, "for range", optimal.range.toFixed(2));
console.log("drag factor 1/m:", dragFactor.toExponential(3));
