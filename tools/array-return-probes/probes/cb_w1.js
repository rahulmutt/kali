function f(){ let s="a"; const g=()=>{ s="b"; }; g(); return s; } console.log(f());
