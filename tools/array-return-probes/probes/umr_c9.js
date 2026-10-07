function mk(){ return {a:1}; } function show(z){ console.log(z.a); } function f(){ let o=mk(); const g=()=>{ show(o); }; g(); } f();
