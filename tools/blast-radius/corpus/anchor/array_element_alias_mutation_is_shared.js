const a = [{ x: 1.5 }, { x: 2.0 }];
const b = a[0];
b.x = b.x + 1.0;
console.log(a[0].x.toFixed(1));
