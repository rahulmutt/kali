const a = new Array(5);
for (let i = 0; i < 5; i = i + 1) { a[i] = i * i; }
let s = 0;
for (let i = 0; i < 5; i = i + 1) { s = s + a[i]; }
console.log(s);
