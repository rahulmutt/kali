class C{ constructor(){ this.n=1; } get(){ return this.n; } } function f(x){ return x.get(); } const g=f; console.log(g(new C()));
