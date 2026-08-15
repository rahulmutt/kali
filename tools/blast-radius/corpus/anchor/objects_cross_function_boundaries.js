function mk(v) { return { x: v }; }
function getx(p) { return p.x; }
const a = mk(3.5);
console.log(getx(a).toFixed(1));
