//! Static recognition and constant-folding of JS/host intrinsic call shapes.
use crate::*;
mod string;
mod array;
pub(crate) use string::{quote_string_literal, strip_string_delimiters};
