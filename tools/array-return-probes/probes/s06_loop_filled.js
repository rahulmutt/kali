function f(n) { const a = new Array(n).fill(0); for (let i = 0; i < n; i++) a[i] = i * i; return a; } function main() { const a = f(4); console.log(a[3]); } main();
