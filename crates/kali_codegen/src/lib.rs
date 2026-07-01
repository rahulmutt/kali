//! WASM code generation for the Kali compiler.

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
    ControlFlowLabelKind, EmittedValue, FunctionEmitter, FunctionPlan, LoopFrame,
    ObjectEnumerationMode, ValueShape,
};
pub(crate) use intrinsics::{
    is_supported_static_ascii_char_code, parse_number_literal, parse_numeric_literal_value,
    quote_string_literal, strip_string_delimiters,
};

use std::{
    borrow::Cow,
    collections::{BTreeMap, HashSet},
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
const COVERAGE_HIT_IMPORT_INDEX: u32 = 20;
const FUNCTION_INDEX_OFFSET: u32 = 20;
const ENV_GET_BUFFER_RESERVED: u32 = 4096;
const STRING_HANDLE_TAG: u64 = 0x8000_0000_0000_0000;

pub use lower::lower_lir_to_wasm;
pub(crate) use lower::{
    emit_literal, encode_string_handle, is_binary_operator_text, is_function_like,
    process_env_property_key,
};

#[cfg(test)]
#[path = "test_support.rs"]
mod test_support;
