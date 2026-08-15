// A turnstile as an explicit finite state machine: feed it an event log, get
// the state trace, the revenue and the rejected pushes. Access-control gates
// are specified this way because the table is the specification.

const STATES = ["locked", "unlocked", "maintenance"];
const FARE = 250;

function transition(state, event) {
  let next = state;
  let effect = "";
  switch (state) {
    case "locked":
      if (event === "coin") {
        next = "unlocked";
        effect = "accept-coin";
      } else if (event === "push") {
        effect = "alarm";
      } else if (event === "service") {
        next = "maintenance";
        effect = "open-panel";
      }
      break;
    case "unlocked":
      if (event === "push") {
        next = "locked";
        effect = "pass";
      } else if (event === "coin") {
        effect = "refund";
      } else if (event === "service") {
        next = "maintenance";
        effect = "open-panel";
      }
      break;
    default:
      if (event === "service") {
        next = "locked";
        effect = "close-panel";
      } else {
        effect = "ignored";
      }
      break;
  }

  if (effect === "") {
    console.warn("unhandled event '" + event + "' in state '" + state + "'");
    effect = "ignored";
  }
  return { state: next, effect: effect };
}

function run(events, start = "locked") {
  let state = start;
  const trace = [];
  let revenue = 0;
  let alarms = 0;
  let passes = 0;
  for (const event of events) {
    const step = transition(state, event);
    if (step.effect === "accept-coin") revenue += FARE;
    if (step.effect === "alarm") alarms += 1;
    if (step.effect === "pass") passes += 1;
    trace.push(state + " --" + event + "--> " + step.state + " [" + step.effect + "]");
    state = step.state;
  }
  return { state: state, trace: trace, revenue: revenue, alarms: alarms, passes: passes };
}

const EVENTS = [
  "push", "coin", "push", "coin", "coin", "push",
  "service", "coin", "service", "push", "coin", "push",
];

const result = run(EVENTS);
for (const line of result.trace) {
  console.log(line);
}

console.log("final state:", result.state);
console.log("revenue (cents):", result.revenue);
console.log("passes:", result.passes, "alarms:", result.alarms);
console.log("ended in a known state:", STATES.indexOf(result.state) >= 0);
console.log("every event consumed:", result.trace.length === EVENTS.length);
