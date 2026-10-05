function m(){ let k=0; while(k<2){ const v=k*5; queueMicrotask(()=>console.log(v)); k++; } } m();
