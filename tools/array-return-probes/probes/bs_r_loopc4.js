function m(){ let f0; for(let i=0;i<3;i++){ const k=i*2; const g=()=>k; if(i===0) f0=g; } return f0(); } console.log(m());
