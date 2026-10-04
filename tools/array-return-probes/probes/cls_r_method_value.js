class C{ constructor(){ this.n=1; } get(){ return this.n; } } const c=new C(); const m=c.get; console.log(typeof m);
