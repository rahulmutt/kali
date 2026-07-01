//! Shared integer-vs-float representation model for the `number` type.
//!
//! Every `number`-typed program point is `I64` unless the representation
//! inference in `kali_types` unifies it with a float seed. The resulting
//! `ReprTable` is threaded to codegen, which uses it to pick wasm signatures,
//! locals, and per-operand arithmetic instructions.

use std::collections::{HashMap, HashSet};

/// Machine representation chosen for a `number` value.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Repr {
    /// Two's-complement 64-bit integer (the default for every `number`).
    #[default]
    I64,
    /// IEEE-754 double.
    F64,
}

/// Representation decisions for a whole program, keyed by function + binding.
///
/// All lookups default to [`Repr::I64`]; only float decisions are stored, so an
/// empty table means "no floats anywhere" and codegen can keep its i64 fast path.
#[derive(Clone, Debug, Default)]
pub struct ReprTable {
    scalars: HashMap<(String, String), Repr>,
    array_elements: HashMap<(String, String), Repr>,
    returns: HashMap<String, Repr>,
    params: HashMap<(String, usize), Repr>,
    /// `(func, binding)` pairs the inference treated as arrays (subscripted,
    /// `.length`/`.fill`, `new Array`, array literal, or a pass-through array
    /// param). Lets codegen register array PARAMETERS as array bindings — an i64
    /// array's element repr is unset (== default I64), so this is the only way to
    /// distinguish an i64 array param from a scalar param.
    array_bindings: HashSet<(String, String)>,
    any_float: bool,
}

impl ReprTable {
    pub fn scalar(&self, func: &str, binding: &str) -> Repr {
        self.scalars
            .get(&(func.to_string(), binding.to_string()))
            .copied()
            .unwrap_or_default()
    }

    pub fn array_element(&self, func: &str, binding: &str) -> Repr {
        self.array_elements
            .get(&(func.to_string(), binding.to_string()))
            .copied()
            .unwrap_or_default()
    }

    pub fn return_repr(&self, func: &str) -> Repr {
        self.returns.get(func).copied().unwrap_or_default()
    }

    pub fn param(&self, func: &str, index: usize) -> Repr {
        self.params
            .get(&(func.to_string(), index))
            .copied()
            .unwrap_or_default()
    }

    pub fn set_scalar(&mut self, func: &str, binding: &str, repr: Repr) {
        if repr == Repr::F64 {
            self.any_float = true;
        }
        self.scalars
            .insert((func.to_string(), binding.to_string()), repr);
    }

    pub fn set_array_element(&mut self, func: &str, binding: &str, repr: Repr) {
        if repr == Repr::F64 {
            self.any_float = true;
        }
        self.array_elements
            .insert((func.to_string(), binding.to_string()), repr);
    }

    pub fn set_return(&mut self, func: &str, repr: Repr) {
        if repr == Repr::F64 {
            self.any_float = true;
        }
        self.returns.insert(func.to_string(), repr);
    }

    pub fn set_param(&mut self, func: &str, index: usize, repr: Repr) {
        if repr == Repr::F64 {
            self.any_float = true;
        }
        self.params.insert((func.to_string(), index), repr);
    }

    /// Record that `(func, binding)` is an array (any element repr). Additive;
    /// does not affect [`is_empty`](Self::is_empty) (an all-integer program with
    /// arrays still has no float decisions).
    pub fn set_array_binding(&mut self, func: &str, binding: &str) {
        self.array_bindings
            .insert((func.to_string(), binding.to_string()));
    }

    /// True when `(func, binding)` was recorded as an array by the inference.
    /// Defaults to false, so a scalar binding/param reports false.
    pub fn is_array_binding(&self, func: &str, binding: &str) -> bool {
        self.array_bindings
            .contains(&(func.to_string(), binding.to_string()))
    }

    /// True when no float representation was ever recorded.
    pub fn is_empty(&self) -> bool {
        !self.any_float
    }
}

/// Disjoint-set forest whose sets carry a sticky "is float" bit.
#[derive(Default)]
pub struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u32>,
    float: Vec<bool>,
}

impl UnionFind {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate a new singleton node and return its id.
    pub fn fresh(&mut self) -> usize {
        let id = self.parent.len();
        self.parent.push(id);
        self.rank.push(0);
        self.float.push(false);
        id
    }

    pub fn find(&mut self, x: usize) -> usize {
        let mut root = x;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        // Path compression.
        let mut cur = x;
        while self.parent[cur] != cur {
            let next = self.parent[cur];
            self.parent[cur] = root;
            cur = next;
        }
        root
    }

    pub fn union(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb {
            return;
        }
        let float = self.float[ra] || self.float[rb];
        let root = if self.rank[ra] < self.rank[rb] {
            self.parent[ra] = rb;
            rb
        } else if self.rank[ra] > self.rank[rb] {
            self.parent[rb] = ra;
            ra
        } else {
            self.parent[rb] = ra;
            self.rank[ra] += 1;
            ra
        };
        self.float[root] = float;
    }

    pub fn seed_float(&mut self, x: usize) {
        let r = self.find(x);
        self.float[r] = true;
    }

    pub fn is_float(&mut self, x: usize) -> bool {
        let r = self.find(x);
        self.float[r]
    }
}

#[cfg(test)]
#[path = "repr_tests.rs"]
mod repr_tests;
