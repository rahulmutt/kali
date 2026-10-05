function f(k){ const g=()=>{ k=k*2; return k; }; const a=g(); return a+k; } console.log(f(3));
