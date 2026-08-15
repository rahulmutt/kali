const p = { x: 1.0, y: 2.5 };
p.x = 4.0;
const q = p;
console.log((q.x + q.y).toFixed(1));
