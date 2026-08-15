const a = new Array(2);
a[0] = 7;
a[1] = 9;
const t = a[0];
a[0] = a[1];
a[1] = t;
console.log(a[0]);
console.log(a[1]);
