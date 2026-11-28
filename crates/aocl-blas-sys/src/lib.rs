//! Raw FFI bindings to AOCL-BLAS (BLIS).
//!
//! This crate exposes the CBLAS C interface plus BLIS's native API, generated
//! by `bindgen` against the AOCL-installed headers. For a safe, idiomatic
//! API see the [`aocl-blas`](https://docs.rs/aocl-blas) crate.

#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    improper_ctypes,
    clippy::all,
    reason = "bindgen output mirrors the C headers: names, unused items and C-only types are not ours to change"
)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Compile-time guard for the types passed by pointer-to-array or by value (docs/SYS-LAYOUT-AUDIT-20261127.md). `scomplex` /
// `dcomplex` are `{ float real, imag; }` / `{ double real, imag; }` (blis.h): 8 bytes align 4, 16 bytes align 8, no alignment
// attribute. `rntm_t` is `BLIS_ATTRIB_ALIGN` = `__attribute__((aligned(64)))` (on `_WIN32` the macro is non-empty only for clang;
// the static BLIS objects reference LLVM's `__kmpc_*` OpenMP entry points, so they are clang-built): the 64-byte alignment is what
// bindgen could silently drop, so it is asserted. Its SIZE is whatever the headers bindgen read say: on `_WIN32` C `bool` is
// `typedef gint_t bool` and `dim_t` is `gint_t`, so `rntm_s` follows `BLIS_INT_TYPE_SIZE`. Measured with clang on AOCL 6.0.68's
// blis.h (LP64 and ILP64 headers, both gint_t = 8 there): size 192, align 64, offsets num_threads 8, thrloop 16, pack_a 64, sba_pool 96,
// info_value 128; the same header text compiled with `-DBLIS_INT_TYPE_SIZE=32 -DBLIS_BLAS_INT_SIZE=32` (gint_t = 4): size 128, align 64,
// num_threads 4, thrloop 8, pack_a 32, sba_pool 48, info_value 72. A hard-coded 192 broke the build on a 32-bit-int BLIS (cpufish#37),
// and another BLIS version may add or drop fields, so the build only requires a multiple of the alignment; tests/rntm_size.rs checks
// the measured per-`gint_t` sizes on the installed headers, and tests/rntm_layout.rs has the library fill one (feature `static-link`).
const _: () = {
    assert!(core::mem::size_of::<scomplex>() == 8);
    assert!(core::mem::align_of::<scomplex>() == 4);
    assert!(core::mem::size_of::<dcomplex>() == 16);
    assert!(core::mem::align_of::<dcomplex>() == 8);
    assert!(core::mem::align_of::<rntm_t>() == 64);
    assert!(core::mem::size_of::<rntm_t>() % 64 == 0);
};
