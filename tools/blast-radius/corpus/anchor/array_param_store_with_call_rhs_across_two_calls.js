function A(i,j){ return 1 / (i + j + 1); }
function Au(u, v) { for (let i = 0; i < u.length; i = i + 1) { v[i] = A(i, 0); } }
function Atu(u, v) { for (let i = 0; i < u.length; i = i + 1) { v[i] = u[i]; } }
function AtAu(u, v, w) { Au(u, w); Atu(w, v); }
const u = new Array(2).fill(1);
const v = new Array(2);
const w = new Array(2);
AtAu(u, v, w);
console.log(v[0] > 0);
