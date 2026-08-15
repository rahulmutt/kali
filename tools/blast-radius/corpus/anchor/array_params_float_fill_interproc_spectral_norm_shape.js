function store(v){v[0]=1/2;}
const u=new Array(3).fill(1);
store(u);
console.log(u[1] > u[0]);
