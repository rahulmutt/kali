const f = () => { console.log("ran"); return [1,2]; }; function g(x){return x[1];} console.log(g(f()));
