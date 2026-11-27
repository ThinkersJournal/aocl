// SPDX-FileCopyrightText: Copyright 2024-2026 Arm Limited and/or its affiliates <open-source-office@arm.com>
// SPDX-FileCopyrightText: Copyright (c) 2026 Advanced Micro Devices, Inc. (AMD fork, AOCL-OpenRNG 6.0.0)
// SPDX-License-Identifier: MIT OR Apache-2.0 WITH LLVM-exception
//! Raw FFI declarations for AOCL-OpenRNG, derived from the public header `include/openrng.h` of
//! `github.com/amd/openrng` tag `6.0.0` (commit `5c05f2eeb6effc696b0df6ce8be1fcbe855b631b`, header sha256
//! `ee3a48fcb50a9a3ded3300bfd342b8a7c7bf8b534f38233be93b26ced577386b`), which is licensed `MIT OR Apache-2.0 WITH
//! LLVM-exception` by Arm Limited (AMD's fork: "AMD contributions MIT"). Unlike this repository's other `-sys` crates, this
//! one is derived from a **licensed public header** (see `NOTICE`: the legal line that covers the guide-derived crates does
//! NOT apply here). The declarations and constants are generated mechanically; provenance per item (header line, tag,
//! commit, sha256) is `docs/cleanroom/openrng/provenance.csv`.
//!
//! **MEASURED on Windows x64 (docs/cleanroom/openrng/MEASUREMENT.md; run by the PM session on CireSnave's 524: A, openrng built at 5c05f2ee, AOCL-LibM from AOCL 6.0.68)**: all 60 symbols link (LP64 and ILP64 builds), `openrng_int_t` is 4 bytes (LP64) / 8 bytes (ILP64),
//! PHILOX4X32X10 outputs recorded. **The `ilp64` feature must match the library build** (a mismatch is silent; see
//! `build.rs`). Not measured: Linux, other compilers, OpenRNG's own tests, bit-reproducibility vs oneMKL.

#![allow(non_camel_case_types, non_snake_case, dead_code)]

/// Header: `typedef int32_t openrng_int_t;` (or `long long` under `OPENRNG_ILP64` / `INTEGER64`).
#[cfg(not(feature = "ilp64"))]
pub type openrng_int_t = i32;
/// Header: `typedef long long openrng_int_t;` under `OPENRNG_ILP64` / `INTEGER64`.
#[cfg(feature = "ilp64")]
pub type openrng_int_t = core::ffi::c_longlong;
/// Header: `typedef uint32_t openrng_uint_t;` (or `unsigned long long` under ILP64).
#[cfg(not(feature = "ilp64"))]
pub type openrng_uint_t = u32;
/// Header: `typedef unsigned long long openrng_uint_t;` under ILP64.
#[cfg(feature = "ilp64")]
pub type openrng_uint_t = core::ffi::c_ulonglong;
/// Header: `typedef unsigned long long openrng_uint64_t;`
pub type openrng_uint64_t = core::ffi::c_ulonglong;
/// Header: `typedef void *VSLStreamStatePtr;`
pub type VSLStreamStatePtr = *mut core::ffi::c_void;

/// Header: `typedef int (*InitStreamPtr)(int method, VSLStreamStatePtr stream, int n, const unsigned int params[]);`
pub type InitStreamPtr = Option<
    unsafe extern "C" fn(
        core::ffi::c_int,
        VSLStreamStatePtr,
        core::ffi::c_int,
        *const core::ffi::c_uint,
    ) -> core::ffi::c_int,
>;
/// Header: `typedef int (*sBRngPtr)(VSLStreamStatePtr stream, int n, float r[], float a, float b);`
pub type sBRngPtr = Option<
    unsafe extern "C" fn(
        VSLStreamStatePtr,
        core::ffi::c_int,
        *mut core::ffi::c_float,
        core::ffi::c_float,
        core::ffi::c_float,
    ) -> core::ffi::c_int,
>;
/// Header: `typedef int (*dBRngPtr)(VSLStreamStatePtr stream, int n, double r[], double a, double b);`
pub type dBRngPtr = Option<
    unsafe extern "C" fn(
        VSLStreamStatePtr,
        core::ffi::c_int,
        *mut core::ffi::c_double,
        core::ffi::c_double,
        core::ffi::c_double,
    ) -> core::ffi::c_int,
>;
/// Header: `typedef int (*iBRngPtr)(VSLStreamStatePtr stream, int n, unsigned int r[]);`
pub type iBRngPtr = Option<
    unsafe extern "C" fn(
        VSLStreamStatePtr,
        core::ffi::c_int,
        *mut core::ffi::c_uint,
    ) -> core::ffi::c_int,
>;
/// Header: `typedef int (*iUpdateFuncPtr)(VSLStreamStatePtr stream, int *n, unsigned int ibuf[], int *nmin, int *nmax, int *idx);`
pub type iUpdateFuncPtr = Option<
    unsafe extern "C" fn(
        VSLStreamStatePtr,
        *mut core::ffi::c_int,
        *mut core::ffi::c_uint,
        *mut core::ffi::c_int,
        *mut core::ffi::c_int,
        *mut core::ffi::c_int,
    ) -> core::ffi::c_int,
>;
/// Header: `typedef int (*dUpdateFuncPtr)(VSLStreamStatePtr stream, int *n, double dbuf[], int *nmin, int *nmax, int *idx);`
pub type dUpdateFuncPtr = Option<
    unsafe extern "C" fn(
        VSLStreamStatePtr,
        *mut core::ffi::c_int,
        *mut core::ffi::c_double,
        *mut core::ffi::c_int,
        *mut core::ffi::c_int,
        *mut core::ffi::c_int,
    ) -> core::ffi::c_int,
>;
/// Header: `typedef int (*sUpdateFuncPtr)(VSLStreamStatePtr stream, int *n, float sbuf[], int *nmin, int *nmax, int *idx);`
pub type sUpdateFuncPtr = Option<
    unsafe extern "C" fn(
        VSLStreamStatePtr,
        *mut core::ffi::c_int,
        *mut core::ffi::c_float,
        *mut core::ffi::c_int,
        *mut core::ffi::c_int,
        *mut core::ffi::c_int,
    ) -> core::ffi::c_int,
>;

/// Header: `typedef struct { int StreamStateSize; int NSeeds; int IncludesZero; int WordSize; int NBits; InitStreamPtr InitStream; sBRngPtr sBRng; dBRngPtr dBRng; iBRngPtr iBRng; } VSLBRngProperties;`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VSLBRngProperties {
    pub StreamStateSize: core::ffi::c_int,
    pub NSeeds: core::ffi::c_int,
    pub IncludesZero: core::ffi::c_int,
    pub WordSize: core::ffi::c_int,
    pub NBits: core::ffi::c_int,
    pub InitStream: InitStreamPtr,
    pub sBRng: sBRngPtr,
    pub dBRng: dBRngPtr,
    pub iBRng: iBRngPtr,
}

mod consts;
mod decls;
pub use consts::*;
pub use decls::*;
