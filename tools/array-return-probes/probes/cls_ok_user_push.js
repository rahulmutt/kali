class Stack{ constructor(){ this.n=0; } push(v){ this.n=this.n+v; return this.n; } } const s=new Stack(); const a=[1]; a.push(2); console.log(a.length, s.push(5));
