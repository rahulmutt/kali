function mk(){ return {a:1, s:"xy", arr:[1,2]}; } function f(){ let o=mk(); const g=()=>o["a"]; return g(); } console.log(f());
