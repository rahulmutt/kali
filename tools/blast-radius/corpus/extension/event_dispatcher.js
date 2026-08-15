// A tiny synchronous event dispatcher with once-handlers and a wildcard
// listener, driven by a scripted event log. Anything with plugins grows one of
// these within a week.

function createDispatcher() {
  const listeners = {};

  function on(event, handler) {
    if (listeners[event] === undefined) {
      listeners[event] = [];
    }
    listeners[event].push({ handler: handler, once: false });
  }

  function once(event, handler) {
    if (listeners[event] === undefined) {
      listeners[event] = [];
    }
    listeners[event].push({ handler: handler, once: true });
  }

  function emit(event, payload) {
    let delivered = 0;
    for (const name of [event, "*"]) {
      const bucket = listeners[name];
      if (bucket === undefined) continue;
      const survivors = [];
      for (const entry of bucket) {
        const handler = entry.handler;
        handler(payload, event);
        delivered += 1;
        if (!entry.once) survivors.push(entry);
      }
      listeners[name] = survivors;
    }
    return delivered;
  }

  function listenerCount(event) {
    return listeners[event] === undefined ? 0 : listeners[event].length;
  }

  return { on: on, once: once, emit: emit, listenerCount: listenerCount };
}

const bus = createDispatcher();
const seen = [];

bus.on("build:start", function (payload) {
  console.log("build starting for " + payload.target);
});

bus.on("build:finish", function (payload) {
  console.log("build finished in " + payload.ms + "ms");
});

bus.once("build:finish", function (payload) {
  console.log("first finish only, exit code " + payload.code);
});

bus.on("*", (payload, event) => seen.push(event));

const SCRIPT = [
  { event: "build:start", payload: { target: "wasm32" } },
  { event: "build:finish", payload: { ms: 631, code: 0 } },
  { event: "build:start", payload: { target: "native" } },
  { event: "build:finish", payload: { ms: 821, code: 0 } },
  { event: "build:cancel", payload: { reason: "user" } },
];

let deliveries = 0;
SCRIPT.forEach(function (step) {
  deliveries += bus.emit(step.event, step.payload);
});

console.log("deliveries:", deliveries);
console.log("events observed by the wildcard:", seen.join(", "));
console.log("finish handlers remaining:", bus.listenerCount("build:finish"));
console.log("the once handler is gone:", bus.listenerCount("build:finish") === 1);
