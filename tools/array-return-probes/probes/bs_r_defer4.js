function m(){ for(const x of [5,6]){ queueMicrotask(()=>console.log(x)); } } m();
