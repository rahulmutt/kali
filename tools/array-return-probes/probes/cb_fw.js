function f(){ let x=1.5; const g=()=>{ x=x+1.25; }; g(); return x; } console.log(f());
