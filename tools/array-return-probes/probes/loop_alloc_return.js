function f() { for (let i = 0; i < 3; i++) { const a = new Array(2).fill(i); if (i === 2) { return a; } } return new Array(1).fill(0); } console.log(f()[0]);
