function m(){ for(let i=0;i<3;i++){ queueMicrotask(()=>console.log(i)); if(i===1) return 7; } } function k(){ let c=0; const inc=()=>{c+=1;}; inc(); console.log(m(), c); } k();
