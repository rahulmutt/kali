function m(){ let a=1; function mid(){ let b=2; queueMicrotask(()=>console.log(a+b)); } mid(); } m();
