const f = () => [1,2,3]; const h = f; function g(x){return x[1];} console.log(g(h()));
