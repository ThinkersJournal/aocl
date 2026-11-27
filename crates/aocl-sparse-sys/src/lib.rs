//! Raw FFI bindings to AOCL-Sparse.
//!
//! For a safe, idiomatic API see [`aocl-sparse`](https://docs.rs/aocl-sparse).

#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    dead_code,
    improper_ctypes,
    clippy::all,
    reason = "bindgen output mirrors the C headers: names, unused items and C-only types are not ours to change"
)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Compile-time guard against the header (aoclsparse_types.h): `aoclsparse_double_complex` is
// `alignas(2 * sizeof(double))` = 16 bytes aligned, size 16; `aoclsparse_float_complex` is `alignas(2 * sizeof(float))` = 8,
// size 8. Both are passed by value; build.rs re-applies the alignment bindgen loses (the header defines `alignas` away in C).
const _: () = {
    assert!(core::mem::size_of::<aoclsparse_double_complex>() == 16);
    assert!(core::mem::align_of::<aoclsparse_double_complex>() == 16);
    assert!(core::mem::size_of::<aoclsparse_float_complex>() == 8);
    assert!(core::mem::align_of::<aoclsparse_float_complex>() == 8);
};
