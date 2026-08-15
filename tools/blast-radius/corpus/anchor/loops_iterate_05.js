function s(n) { if (n < 1) { return 0; } return n + s(n - 1); }
console.log(s(5));
