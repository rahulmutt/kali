class A{ f(){return 4;} } class B extends A{ g(){ return 1; } } const b=new B(); console.log(b.f()+b.g());
