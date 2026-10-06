function f(k){ let n=k; const g=()=>n; console.log(n); return g(); } console.log(f(5));
