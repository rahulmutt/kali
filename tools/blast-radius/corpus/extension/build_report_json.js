// Turn raw per-target build timings into the JSON envelope a CI job uploads:
// derive the aggregate fields, freeze the finished document so a later stage
// cannot mutate it, and print it.

const TIMINGS = [
  { target: "wasm32", phase: "parse", ms: 41 },
  { target: "wasm32", phase: "check", ms: 188 },
  { target: "wasm32", phase: "codegen", ms: 402 },
  { target: "native", phase: "parse", ms: 39 },
  { target: "native", phase: "check", ms: 175 },
  { target: "native", phase: "codegen", ms: 511 },
  { target: "native", phase: "link", ms: 96 },
];

function byTarget(timings) {
  const targets = {};
  for (const timing of timings) {
    if (targets[timing.target] === undefined) {
      targets[timing.target] = { phases: {}, totalMs: 0, slowestPhase: "" };
    }
    const entry = targets[timing.target];
    entry.phases[timing.phase] = timing.ms;
    entry.totalMs += timing.ms;
    if (entry.slowestPhase === "" || timing.ms > entry.phases[entry.slowestPhase]) {
      entry.slowestPhase = timing.phase;
    }
  }
  return targets;
}

function deepFreeze(value) {
  if (value === null || typeof value !== "object") return value;
  for (const key of Object.keys(value)) {
    deepFreeze(value[key]);
  }
  Object.freeze(value);
  return value;
}

function buildReport(timings, schema = "kali.build.timings.v1") {
  const targets = byTarget(timings);
  const names = Object.keys(targets).sort();
  let wallMs = 0;
  for (const name of names) {
    wallMs += targets[name].totalMs;
  }
  return {
    schema: schema,
    generatedBy: "build_report_json.js",
    targets: targets,
    summary: {
      targetCount: names.length,
      phaseCount: timings.length,
      wallMs: wallMs,
      meanPhaseMs: Math.round(wallMs / timings.length),
    },
  };
}

const report = deepFreeze(buildReport(TIMINGS));

console.log(JSON.stringify(report, null, 2));
console.log("frozen:", Object.isFrozen(report));

const summary = report.summary;
console.log(summary);
console.log("compact:", JSON.stringify(summary));

for (const target of Object.keys(report.targets).sort()) {
  console.log(target + " slowest phase: " + report.targets[target].slowestPhase);
}
