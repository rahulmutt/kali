async function m(){ for(let i=0;i<2;i++){ await null; setTimeout(()=>console.log(i),0); } } m();
