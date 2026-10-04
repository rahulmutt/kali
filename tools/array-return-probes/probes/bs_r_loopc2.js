function m(){ let f0; let f2; for(let i=0;i<3;i++){ const g=()=>i; if(i===0) f0=g; if(i===2) f2=g; } return f0()*10+f2(); } console.log(m());
