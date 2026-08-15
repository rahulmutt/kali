function store(v){v[0]=1/2;}
function get(v){return v[0];}
const u=new Array(2);
store(u);
console.log(get(u) < 1);
