function mk(){ return {a:1}; } function f(){ let o=mk(); const g=()=>o; return g().a; } console.log(f());
