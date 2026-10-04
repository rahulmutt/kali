class A{ constructor(){ this.n=1; } } class B{ constructor(){ this.n=2; } } function f(x){ return x.n; } console.log(f(new A()), f(new B()));
