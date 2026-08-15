function half(i){ return 1 / (i + 2); }
function fillIt(v){ for (let i = 0; i < v.length; i = i + 1) { v[i] = half(i); } }
const u = new Array(2);
fillIt(u);
console.log(u[0] < 1);
