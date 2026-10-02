function mk(x) { return x; } function main() { const a = mk(new Array(2).fill(3)); const r = [1, 2].map(mk); console.log(a[0] + "," + r[1]); } main();
