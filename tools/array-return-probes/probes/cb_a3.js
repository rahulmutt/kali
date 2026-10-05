function f(k){ const g=()=>k; k=k+1; return g(); } console.log(f(5));
