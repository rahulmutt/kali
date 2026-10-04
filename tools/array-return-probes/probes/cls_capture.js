class C{ constructor(){ this.n=0; } add(x){ this.n=this.n+x; } } const s=new C(); const f = () => { s.add(4); }; f(); console.log(s.n);
