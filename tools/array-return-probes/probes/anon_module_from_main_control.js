const f = () => [1,2,3]; function main(){ function g(x){return x[1];} console.log(g(f())); } main();
