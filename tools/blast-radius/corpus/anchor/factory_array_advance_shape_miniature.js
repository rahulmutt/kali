function mk(x, vx) { return { x: x, vx: vx }; }
function advance(bs, dt) {
for (let i = 0; i < bs.length; i = i + 1) {
const b = bs[i];
b.x = b.x + dt * b.vx;
}
}
const bs = [mk(1.0, 2.0), mk(0.5, 4.0)];
advance(bs, 0.5);
console.log((bs[0].x + bs[1].x).toFixed(2));
