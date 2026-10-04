class M{ constructor(){ this.calls=0; } fact(n){ this.calls=this.calls+1; if (n<=1) { return 1; } return n*this.fact(n-1); } } const m=new M(); console.log(m.fact(5), m.calls);
