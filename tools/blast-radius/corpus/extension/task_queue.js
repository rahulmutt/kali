// Run a queue of jobs in dependency order with bounded retries and optional
// progress hooks. Build scripts grow one of these the first time a step has to
// wait for another.

const JOBS = [
  { name: "fetch", needs: [], flakyUntil: 2, cost: 3 },
  { name: "parse", needs: ["fetch"], flakyUntil: 0, cost: 5 },
  { name: "check", needs: ["parse"], flakyUntil: 1, cost: 8 },
  { name: "codegen", needs: ["check"], flakyUntil: 0, cost: 13 },
  { name: "link", needs: ["codegen", "fetch"], flakyUntil: 0, cost: 4 },
  { name: "docs", needs: ["parse"], flakyUntil: 0, cost: 2 },
];

function topologicalOrder(jobs) {
  const done = {};
  let queue = [];
  let remaining = jobs.slice();
  while (remaining.length > 0) {
    const ready = remaining.filter((job) => job.needs.every((need) => done[need] === true));
    if (ready.length === 0) {
      console.warn("dependency cycle among " + remaining.map((job) => job.name).join(", "));
      break;
    }
    for (const job of ready) {
      done[job.name] = true;
      queue.push(job);
    }
    remaining = remaining.filter((job) => done[job.name] !== true);
  }
  return queue;
}

function attemptJob(job, attempt) {
  return attempt > job.flakyUntil;
}

function runQueue(jobs, hooks = {}, maxAttempts = 4) {
  const order = topologicalOrder(jobs);
  const failures = [];
  let elapsed = 0;
  let retries = 0;

  for (const job of order) {
    hooks.onStart?.(job.name);
    for (let attempt = 1; ; attempt++) {
      elapsed += job.cost;
      if (attemptJob(job, attempt)) {
        hooks.onFinish?.(job.name, attempt);
        break;
      }
      retries += 1;
      hooks.onRetry?.(job.name, attempt);
      if (attempt >= maxAttempts) {
        failures.push(job.name);
        break;
      }
    }
  }

  return { order: order, elapsed: elapsed, retries: retries, failures: failures };
}

const hooks = {
  onStart: function (name) {
    console.log("start " + name);
  },
  onRetry: (name, attempt) => console.warn("retry " + name + " after attempt " + attempt),
  onFinish: function (name, attempt) {
    console.log("done  " + name + " (attempts: " + attempt + ")");
  },
};

const result = runQueue(JOBS, hooks);

console.log("order:", result.order.map((job) => job.name).join(" -> "));
console.log("simulated cost:", result.elapsed, "retries:", result.retries);
console.log("failures:", result.failures.length === 0 ? "none" : result.failures.join(", "));

const quiet = runQueue(JOBS);
console.log("hookless run matches:", quiet.elapsed === result.elapsed);
