function main(){ const a=new Array(3).fill(4); a[2]=7; a.copyWithin(0,2); console.log(a[0]); } main();
