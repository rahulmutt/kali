function m() { let a = 5; const o = () => { let z = 10; const h = () => z + a; return h(); }; return o(); } console.log(m());
