class A{ constructor(){ this.n=1; } } class B extends A{ g(){ return this.n; } } const b=new B(); console.log(b.g());
