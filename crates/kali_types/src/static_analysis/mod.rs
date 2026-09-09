//! Static-value analysis used during resolution.
mod array;
/// The one fold rule for a nameless computed member index, shared by the
/// resolver and the materialization pass (`repr_infer`).
pub(crate) mod computed_member;
mod math;
mod number;
mod object;
mod promise;
mod string;
