let f0=null; let f2=null; for(let i=0;i<3;i++){ const g=()=>i; if(i===0) f0=g; if(i===2) f2=g; } console.log(f0(), f2());
