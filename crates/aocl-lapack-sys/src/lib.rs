//! Raw FFI bindings to AOCL-LAPACK (libFLAME).
//!
//! Exposes the LAPACKE C interface and standard LAPACK Fortran-symbol
//! declarations. For a safe, idiomatic API see [`aocl-lapack`](https://docs.rs/aocl-lapack).

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

// Compile-time guard: LAPACKE passes `lapack_complex_float` / `lapack_complex_double` (`float _Complex` / `double _Complex`,
// lapack.h) BY VALUE; bindgen declares them as the generic `__BindgenComplex<T>`. C layout (clang, -x c and C++, Windows x64):
// 8 bytes align 4 and 16 bytes align 8 (docs/SYS-LAYOUT-AUDIT-20261127.md; tests/by_value_complex.rs calls the library).
const _: () = {
    assert!(core::mem::size_of::<__BindgenComplex<f32>>() == 8);
    assert!(core::mem::align_of::<__BindgenComplex<f32>>() == 4);
    assert!(core::mem::size_of::<__BindgenComplex<f64>>() == 16);
    assert!(core::mem::align_of::<__BindgenComplex<f64>>() == 8);
};
