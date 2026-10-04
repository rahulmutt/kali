class C{ constructor(){ this.n=1; } bump(){ const f = () => { this.n = this.n + 1; }; f(); f(); return this.n; } } const c=new C(); console.log(c.bump());
