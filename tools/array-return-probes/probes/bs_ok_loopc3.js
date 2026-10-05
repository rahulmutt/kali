function m(){ let s=0; for(let i=0;i<3;i++){ const g=()=>i; s+=g(); } return s; } console.log(m());
