# Bootstrap Brief

This top-level file mirrors the original bootstrap brief from [`prompts/bootstrap.md`](./prompts/bootstrap.md) so the rest of the spec set can link to one stable root-level path.

Reading rule:
- treat this file as directional input only
- for normalized product claims, read [`SPEC.md`](./SPEC.md)
- for shipped availability, read [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md)
- for current proof-backed scope, read [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md)

---

Repository reading note:
- this file is the original input brief
- after normalization, [`SPEC.md`](./SPEC.md) is the cross-spec source of truth, [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) owns shipped availability, and [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) owns the current verification-claim boundary
- when this brief sounds broader than the detailed spec set, read it through that normalized split rather than as a same-phase MVP guarantee

Write a top-level SPEC.md that references specs/*.md to breakdown into logical units the implementation of the following:

- Kali is an implementation of TypeScript that uses the extra type information to generate straight and fast WebAssembly code from it. It should also support compiling JavaScript with type inference for efficient compilation.
- Kali is designed with sandboxing as a first-class concern, because it is intended as a target for AI agents to generate code from. It should be able to constrain things such as number of processes spawned, CPU usage, memory usage, etc. Sandboxing / validation of core APIs / syscalls can be tightly controlled — users can declare functions that determine conditions under which a core API is valid or not via a sandboxing policy passed in when running a Kali program.
- Should be able to statically run a command and get a JSON output of all the potential effects the program can perform. If needed, extend the type system with effects if that makes it easier to implement this feature and if it will help with sandboxing.
- Kali can be a superset of TypeScript - extend the type system and the type inference algorithm to do more advanced (but fast) type checking. Constraint solving is on the table.
- Take inspiration and best practices from projects like:
 - [Boa](https://github.com/boa-dev/boa)
 - [V8](https://github.com/v8/v8)
 - JavaScriptCore
 - SpiderMonkey
 - [Deno](https://github.com/denoland/deno)
 - [tsc](https://github.com/microsoft/Typescript)
 - [Porffor](https://github.com/CanadaHonk/porffor)
 - [Hermes](https://github.com/facebook/hermes)
 - [Bun](https://github.com/oven-sh/bun)
- No JIT compilation! This project is not designed for Just-In Time compilation at all and it should AOT compile (Ahead of Time).
- No Garbage Collection, must decide at compile-time whether to allocate to the heap, to the stack, similar to Rust. Should also decide whether to use shared references (like Rc<T>) in Rust all at compile-time.
- Aggressively specialize generic functions - specialize both memory layouts of inputs / outputs - based on call-site usage.
- Design intermediate representations to support a blazing fast runtime. The IR should be explicit about memory layouts and when the memory layout of a JS object is unknown / dynamic, resort to inefficient representation, and when it is known / consistent, optimize the memory layout as much as possible. When a dynamic feature is used, it should automatically turn off optimizations unless there's a way to reason about it. Perhaps also have the type system mark objects that are dynamic of this nature. If needed, define several IRs with transformations between them as needed.
- For running webassembly, feel free to use wasmtime or wasmer according to what suits the use case better.
- Have a comprehensive test suite for inspired by the upstream [tsc](https://github.com/microsoft/tsc) implementation. Extend the type inference / system from tsc to add Hindley-Milner like type inference at the same time keeping it efficient. Analyze function bodies to get the "behavior" of each variable as effectively as possible while combining flow type-inference like tsc.
- Lexing, Parsing, Typechecking, webassembly code generation should all be blazing fast. there should be flags / modes to run advanced optimizations if users want to run much faster.
- Must be embeddable - should expose a C API to make it easy to embed from any other language.
- Must be implemented in Rust using the standard best practices.
- Should support Deno API, Node.js API, and browser API.
- Should support the latest [ECMA-262 standard](https://262.ecma-international.org/16.0/index.html)
- Should support non node-gyp packages from `npm` for easy access to millions of existing JavaScript packages.
- Like Deno, should make it easy to use Kali as a Rust library and expose a nice API.
- Support for all features (including `eval`).
- Should also formally verify implementation details with Lean while iterating on the Spec.
- CLI usage should be clean and similar to deno - formatting, linting, typechecking, running, etc.
- Do NOT embed any C / C++ libraries at all. Make everything implemented in Rust as much as possible.
- It should have excellent error messages - AI agents should easily parse the error messages and be able to react easily.
- Design the CLI output for AI agent consumption - don't be too verbose (add verbosity as a separate flag) on successes and for failures provide just enough detail to make AI loops token-efficient.
- Take inspiration in language / type system design from languages like Haskell, Idris, Agda, Lean. At the same time aim for a pragmatic and ergonomic language like Rust.
- Add support for WIT / WebAssembly Component Model (make it a default if it is sensible to do) and keep the interfaces well integrated with sandboxing and Kali-specific.

- Should have a benchmark suite adapted from the Node.js / JavaScript submissions to the [Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/index.html) and adapt the JavaScript examples to TypeScript and use Kali-specific optimizations to be on par with Rust performance on those benchmarks.
- Should have e2e tests that verify that the following npm binaries run and simple commands work:
  - [semver](https://www.npmjs.com/package/semver)
  - [pi-coding-agnet](https://www.npmjs.com/package/@mariozechner/pi-coding-agent)
