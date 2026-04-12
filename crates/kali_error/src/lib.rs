//! Diagnostic system for the Kali compiler.
//!
//! This crate provides:
//! - Error code namespaces for different compiler stages
//! - Diagnostic types and severity levels
//! - Non-aborting diagnostic collection

pub mod diagnostic;
pub mod severity;

pub use diagnostic::Diagnostic;
pub use severity::Severity;

#[doc(hidden)]
pub mod _error_codes {
    // E1xxx: Lex errors (kali_lexer)
    pub mod e1 {
        // E1000-1099: Basic lexing errors
        pub const UNTERMINATED_STRING: u16 = 1000;
        pub const UNTERMINATED_TEMPLATE: u16 = 1001;
        pub const UNEXPECTED_CHARACTER: u16 = 1002;
        pub const ILLEGAL_BACKSLASH: u16 = 1003;

        // E1100-1199: Number parsing errors
        pub const INVALID_NUMBER: u16 = 1100;
        pub const OVERFLOW_IN_NUMBER: u16 = 1101;

        // E1200-1299: Identifier errors
        pub const ILLEGAL_SYMBOL: u16 = 1200;
    }

    // E2xxx: Parse errors (kali_parser)
    pub mod e2 {
        // E2000-2099: Syntax errors
        pub const EXPECTED_TOKEN: u16 = 2000;
        pub const UNEXPECTED_TOKEN: u16 = 2001;
        pub const MISSING_ITEM: u16 = 2002;
        pub const DUPLICATE_ITEM: u16 = 2003;

        // E2100-2199: Parse state errors
        pub const RECOVERY_FAILED: u16 = 2100;

        // E2200-2299: Unexpected end of input
        pub const UNEXPECTED_EOF: u16 = 2200;
    }

    // E3xxx: Type/name resolution errors (early stages)
    // Note: E3xxx used by kali_types for basic type errors
    pub mod e3 {
        // E3000-3099: Import resolution
        pub const IMPORT_NOT_FOUND: u16 = 3000;
        pub const UNRESOLVED_SPECIFIER: u16 = 3001;
        pub const INVALID_IMPORT_PATH: u16 = 3002;

        // E3100-3199: Binding errors
        pub const UNDEFINED_IDENTIFIER: u16 = 3100;
        pub const DUPLICATE_BINDING: u16 = 3101;

        // E3200-3299: Type errors (basic)
        pub const TYPE_MISMATCH: u16 = 3200;
        pub const MISSING_PARAMETER_TYPE: u16 = 3201;
    }

    // E4xxx: Runtime errors (kali_runtime)
    pub mod e4 {
        // E4000-4099: Execution errors
        pub const UNCAUGHT_ERROR: u16 = 4000;
        pub const STACK_OVERFLOW: u16 = 4001;

        // E4100-4199: Type errors at runtime
        pub const INVALID_TYPE_OPERATION: u16 = 4100;

        // E4200-4299: Resource errors
        pub const RESOURCE_LIMIT_EXCEEDED: u16 = 4200;
        pub const IO_ERROR: u16 = 4201;
    }

    // E5xxx: CLI/command errors (kali_cli)
    pub mod e5 {
        // E5000-5099: Command argument and availability errors
        pub const INVALID_ARGUMENT: u16 = 5000;
        pub const MISSING_REQUIRED_ARGUMENT: u16 = 5001;
        pub const UNKNOWN_COMMAND: u16 = 5002;
        pub const FEATURE_UNAVAILABLE: u16 = 5006;
        pub const INVALID_PRIMARY_INPUT_KIND: u16 = 5007;
        pub const INVALID_CLI_USAGE: u16 = 5008;

        // E5100-5199: Command mode errors
        pub const INCOMPATIBLE_FLAGS: u16 = 5100;
        pub const INVALID_MODE: u16 = 5101;

        // E5200-5299: Output errors
        pub const OUTPUT_ERROR: u16 = 5200;

        // E5300-5399: Configuration errors
        pub const INVALID_CONFIG: u16 = 5300;
    }

    // E6xxx: Package management errors (kali_npm)
    pub mod e6 {
        // E6000-6099: Resolution errors
        pub const NOT_FOUND: u16 = 6000;
        pub const VERSION_MISMATCH: u16 = 6001;
        pub const LOCK_CONFLICT: u16 = 6002;

        // E6100-6199: Installation errors
        pub const INSTALL_FAILED: u16 = 6100;
        pub const UNRESOLVABLE_DEPENDENCY: u16 = 6101;

        // E6200-6299: Package compatibility
        pub const INCOMPATIBLE_PACKAGE: u16 = 6200;

        // E6300-6399: Lock file errors
        pub const INVALID_LOCK_FILE: u16 = 6300;

        // E6400-6499: Raw URL errors
        pub const INVALID_RAW_URL: u16 = 6400;
    }

    // E7xxx: WASM validation errors (kali_codegen - validator)
    pub mod e7 {
        // E7000-7099: Validation errors
        pub const INVALID_WASM_MODULE: u16 = 7000;
        pub const UNRESERVED_CONSTANT: u16 = 7001;
        pub const TYPE_MISMATCH: u16 = 7002;

        // E7100-7199: Validation errors
        pub const MEMORY_OVERFLOW: u16 = 7100;
        pub const STACK_OVERFLOW: u16 = 7101;

        // E7200-7299: Validation errors
        pub const HOST_EXPORT_MISSING: u16 = 7200;
    }

    // E8xxx: Internal codegen errors (kali_codegen - emitter)
    pub mod e8 {
        // E8000-8099: Codegen internal errors
        pub const CODEGEN_UNEXPECTED: u16 = 8000;
        pub const UNIMPLEMENTED: u16 = 8001;
        pub const IR_UNREADABLE: u16 = 8002;

        // E8100-8199: Internal state errors
        pub const INTERNAL_ERROR: u16 = 8100;
    }

    // E9xxx: Sandbox/policy errors (kali_sandbox)
    pub mod e9 {
        // E9000-9099: Policy errors
        pub const POLICY_VIOLATION: u16 = 9000;
        pub const UNRESOLVED_POLICY: u16 = 9001;

        // E9100-9199: Policy validation
        pub const INVALID_POLICY: u16 = 9100;
        pub const POLICY_CONFLICT: u16 = 9101;

        // E9200-9299: Effects errors
        pub const UNDECLARED_EFFECT: u16 = 9200;

        // E9300-9399: Runtime sandbox enforcement
        pub const SANDBOX_VIOLATION: u16 = 9300;

        // E9400-9499: Static effect analysis
        pub const EFFECT_RESOLUTION_ERROR: u16 = 9400;

        // E9500-9599: Effect vs policy
        pub const EFFECT_POLICY_MISMATCH: u16 = 9500;

        // E9600-9699: Configuration errors
        pub const SANDBOX_CONFIG_INVALID: u16 = 9600;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_namespace_structure() {
        // Verify all namespace modules are accessible
        use _error_codes::*;

        // E1 namespace
        assert_eq!(e1::UNTERMINATED_STRING, 1000);

        // E2 namespace
        assert_eq!(e2::EXPECTED_TOKEN, 2000);

        // E3 namespace
        assert_eq!(e3::UNDEFINED_IDENTIFIER, 3100);

        // E4 namespace
        assert_eq!(e4::UNCAUGHT_ERROR, 4000);

        // E5 namespace
        assert_eq!(e5::UNKNOWN_COMMAND, 5002);

        // E6 namespace
        assert_eq!(e6::NOT_FOUND, 6000);

        // E7 namespace
        assert_eq!(e7::INVALID_WASM_MODULE, 7000);

        // E8 namespace
        assert_eq!(e8::UNIMPLEMENTED, 8001);

        // E9 namespace
        assert_eq!(e9::POLICY_VIOLATION, 9000);
    }

    #[test]
    fn test_diagnostic_creation() {
        let diag = Diagnostic::new(Severity::Error, 1000, "test error".to_string());

        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.code, Some(1000));
    }
}
