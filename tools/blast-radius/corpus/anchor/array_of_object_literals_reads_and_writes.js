const a = [{ x: 1.0 }, { x: 2.0 }];
a[1].x = 5.0;
console.log((a[0].x + a[1].x).toFixed(1));
