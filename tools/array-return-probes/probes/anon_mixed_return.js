const f = (c) => { if (c) { return [1]; } return 0; }; function g(x){return x[0];} console.log(g(f(true)));
