function m(){ for(let i=0;i<3;i++){ if(i===1) continue; queueMicrotask(()=>console.log(i)); } } m();
