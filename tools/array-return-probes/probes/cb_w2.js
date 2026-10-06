function f(){ let x=1.5; const g=()=>{ x=2.5; }; g(); return x; } console.log(f());
