class C{ constructor(){ this.n=0; } inc(){ this.n++; } get(){ return this.n; } } const s=new C(); s.inc(); s.inc(); console.log(s.get(), s.n);
