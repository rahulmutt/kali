function show(z){ return z.n * 10; } function outer(p){ const obj = p; function rd(){ return show(obj); } console.log(rd()); } const x={n:4}; outer(x);
