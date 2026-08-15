const p = { x: 1.0 };
let q = { x: 2.0 };
q.x = 3.0;
q = p;
console.log(q.x.toFixed(1));
