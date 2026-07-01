// The Computer Language Benchmarks Game
// https://benchmarksgame-team.pages.debian.net/benchmarksgame/
// spectral-norm — idiomatic TS port of the Node.js / JavaScript submission,
// normalized to Kali's pipeline (no intrinsic tuning). Retains upstream attribution.
function A(i, j) {
  return 1 / ((i + j) * (i + j + 1) / 2 + i + 1);
}
function Au(u, v) {
  for (let i = 0; i < u.length; i = i + 1) {
    let t = 0;
    for (let j = 0; j < u.length; j = j + 1) {
      t = t + A(i, j) * u[j];
    }
    v[i] = t;
  }
}
function Atu(u, v) {
  for (let i = 0; i < u.length; i = i + 1) {
    let t = 0;
    for (let j = 0; j < u.length; j = j + 1) {
      t = t + A(j, i) * u[j];
    }
    v[i] = t;
  }
}
function AtAu(u, v, w) {
  Au(u, w);
  Atu(w, v);
}
function spectralnorm(n) {
  const u = new Array(n).fill(1);
  const v = new Array(n);
  const w = new Array(n);
  for (let i = 0; i < 10; i = i + 1) {
    AtAu(u, v, w);
    AtAu(v, u, w);
  }
  let vBv = 0;
  let vv = 0;
  for (let i = 0; i < n; i = i + 1) {
    vBv = vBv + u[i] * v[i];
    vv = vv + v[i] * v[i];
  }
  return Math.sqrt(vBv / vv);
}
console.log(spectralnorm(100).toFixed(9));
