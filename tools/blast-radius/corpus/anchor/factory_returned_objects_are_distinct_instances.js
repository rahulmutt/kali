function mk(v) { return { x: v }; }
const p = mk(1.0);
const q = mk(2.0);
q.x = 5.0;
console.log((p.x + q.x).toFixed(1));
