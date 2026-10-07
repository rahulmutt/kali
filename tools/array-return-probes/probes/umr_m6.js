function f(p){ const o=p; const g=()=>o["a"]; return g(); } const x={a:1}; console.log(f(x));
