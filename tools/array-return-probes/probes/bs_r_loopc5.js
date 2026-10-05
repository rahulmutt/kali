function m(){ let f0; for(const x of [5,6,7]){ const g=()=>x; if(x===5) f0=g; } return f0(); } console.log(m());
