class X extends EventTarget { fire(){ this.addEventListener("t", () => {}); return 1; } } const x = new X(); console.log(x.fire());
