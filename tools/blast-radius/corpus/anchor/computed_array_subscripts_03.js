const a = new Array(4);
for (let i = 0; i < 4; i = i + 1) { a[i] = i; }
let i = 0;
while (i < 3) { a[i] = a[i + 1]; i = i + 1; }
console.log(a[0]);
console.log(a[1]);
console.log(a[2]);
