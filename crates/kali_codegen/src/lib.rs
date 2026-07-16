//! WASM code generation for the Kali compiler.

mod closure;
mod ctx;
mod emit;
mod emitter;
mod intrinsics;
mod lower;
pub use ctx::{CodegenCtx, CodegenResult, TargetConfig};
use ctx::{
    StaticArrayAtResult, StaticArraySearchResult, StaticIndexMemberResult,
    StaticObjectIdentityValue, StaticStringAtResult, StringPool,
};
use emitter::{
    ArenaFrame, ControlFlowLabelKind, EmittedValue, FunctionEmitter, FunctionPlan, LoopFrame,
    ObjectEnumerationMode, ValueShape,
};
pub(crate) use intrinsics::{
    is_supported_static_ascii_char_code, parse_number_literal, parse_numeric_literal_value,
    quote_string_literal, strip_string_delimiters,
};

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    path::PathBuf,
};

use kali_common::generator_function_yield_lowering_unavailable_message;
use kali_error::{
    _error_codes::{e3, e5, e8},
    Diagnostic, DiagnosticContext, DiagnosticContextOrigin,
};
use kali_lir::{FunctionFlavor, LirNode, LirNodeId, LirNodeKind, LirProgram};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use wasm_encoder::{
    BlockType, CodeSection, ConstExpr, CustomSection, DataSection, EntityType, ExportKind,
    ExportSection, Function, FunctionSection, GlobalSection, GlobalType, ImportSection,
    Instruction, MemArg, MemorySection, MemoryType, Module, TypeSection, ValType,
};

const TEST_REGISTER_IMPORT_INDEX: u32 = 0;
const CONSOLE_LOG_IMPORT_INDEX: u32 = 1;
const CONSOLE_ERROR_IMPORT_INDEX: u32 = 2;
const CONSOLE_WARN_IMPORT_INDEX: u32 = 3;
const CONSOLE_INFO_IMPORT_INDEX: u32 = 4;
const CONSOLE_DEBUG_IMPORT_INDEX: u32 = 5;
const ARGS_LEN_IMPORT_INDEX: u32 = 6;
const MATH_MAX_IMPORT_INDEX: u32 = 7;
const MATH_MIN_IMPORT_INDEX: u32 = 8;
const MATH_ABS_IMPORT_INDEX: u32 = 9;
const MATH_SIGN_IMPORT_INDEX: u32 = 10;
const MATH_IMUL_IMPORT_INDEX: u32 = 11;
const MATH_ROUND_IMPORT_INDEX: u32 = 12;
const PROCESS_PID_IMPORT_INDEX: u32 = 13;
const CWD_IMPORT_INDEX: u32 = 14;
const MATH_CLZ32_IMPORT_INDEX: u32 = 15;
const MATH_POW_IMPORT_INDEX: u32 = 16;
const INT_TO_STRING_IMPORT_INDEX: u32 = 17;
const STRING_CONCAT_IMPORT_INDEX: u32 = 18;
const FLOAT_TO_FIXED_IMPORT_INDEX: u32 = 19;
const FLOAT_TO_STRING_IMPORT_INDEX: u32 = 20;
// Current-arena twin of `string_concat` (fasta Spec 7 Task 4d): allocates the
// concat result into the resettable current arena (`__alloc`) instead of the
// never-reset `__alloc_global`. Appended as the LAST always-present import so
// no FIXED import index (0..=20) shifts; only the CONDITIONAL imports that
// follow (coverage_hit, env_*, …) and the function-index base move uniformly
// by +1, and both are referenced solely through the `COVERAGE_HIT_IMPORT_INDEX`
// / `FUNCTION_INDEX_OFFSET` constants below (recomputed here), so the shift is
// mechanical and behavior-neutral. Selected per concat site by
// `FunctionEmitter::string_concat_import_index`.
const STRING_CONCAT_ARENA_IMPORT_INDEX: u32 = 21;
const COVERAGE_HIT_IMPORT_INDEX: u32 = 22;
const FUNCTION_INDEX_OFFSET: u32 = 22;
const ENV_GET_BUFFER_RESERVED: u32 = 4096;
const STRING_HANDLE_TAG: u64 = 0x8000_0000_0000_0000;
/// Tag bit 62 marks a GROWABLE runtime-array handle (throw-fallout Stage 4):
/// `handle = zero_extend(hdr_ptr) | ARRAY_HANDLE_TAG`, where `hdr_ptr` (low
/// 32 bits) addresses the 24-byte header `[len:i64 @+0][cap:i64 @+8]
/// [data_ptr:i64 @+16]`. Distinct from `STRING_HANDLE_TAG` (bit 63) and from
/// the plain-array lane's UNtagged `[len][elem…]` base pointers. Decode:
/// `(handle & !ARRAY_HANDLE_TAG)` then `I32WrapI64` (the string-decode
/// idiom; realloc rewrites `data_ptr` INSIDE the header, so the handle is
/// stable across growth).
pub(crate) const ARRAY_HANDLE_TAG: u64 = 0x4000_0000_0000_0000;

pub use lower::lower_lir_to_wasm;
pub(crate) use lower::{
    emit_literal, encode_string_handle, is_binary_operator_text, is_function_like,
    process_env_property_key,
};

#[cfg(test)]
#[path = "test_support.rs"]
mod test_support;
