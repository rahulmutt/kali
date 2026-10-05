function m(){ for(let i=0;i<2;i++){ const k=i*3; queueMicrotask(()=>console.log(k)); } } m();
