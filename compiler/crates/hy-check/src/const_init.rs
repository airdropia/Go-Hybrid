//! const_init.rs stub — `const` initializer validation.
//! Minimal stub providing `is_conversion_type_name` for buildable.rs.
//! Full implementation is on the author's branch.

/// Is `name` a type that can be used in a compile-time constant conversion?
pub(crate) fn is_conversion_type_name(name: &str) -> bool {
    matches!(
        name,
        "int" | "int8" | "int16" | "int32" | "int64"
            | "uint" | "uint8" | "uint16" | "uint32" | "uint64"
            | "float32" | "float64"
            | "string" | "bool"
            | "complex64" | "complex128"
    )
}
