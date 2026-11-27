// types_decls.rs -- Rust definitions of types the AMD AOCL API Guide 6.0 uses but never defines,
// MEASURED by running the installed 6.0.68 libraries (see TYPES-MEASUREMENT.md). Windows x86_64 (MS x64 ABI) ONLY.
// Linux and the self-hosted rig were NOT measured: nothing below is claimed for them.
// No AMD header was read to produce this file.
#![allow(non_camel_case_types)]
use std::os::raw::{c_double, c_int};

// ---- LibM ------------------------------------------------------------------------------------
/// C `long double` as used by `amd_nexttoward(double x, long double y)` and
/// `amd_nexttowardf(float x, long double y)`. MEASURED: on this target it is a plain 64-bit IEEE
/// double passed in xmm1 (NOT an 80-bit value, NOT passed by reference). Rust: `c_double` (== f64).
pub type c_long_double = c_double;

/// MEASURED: two f32, real part at offset 0, imaginary part at offset 4 (8 bytes).
/// Windows x64: passed in ONE integer register (rcx / rdx for a 2nd arg) and returned in rax --
/// exactly what a Rust `#[repr(C)]` 8-byte struct does under `extern "C"` on this target.
#[repr(C)] #[derive(Clone, Copy, Debug, PartialEq)]
pub struct fc32_t { pub re: f32, pub im: f32 }

/// MEASURED: two f64, real part at offset 0, imaginary part at offset 8 (16 bytes).
/// Windows x64: every argument is passed BY HIDDEN REFERENCE (pointer to a caller copy) and the result
/// is returned via a hidden sret pointer in rcx (rax returns that pointer) -- exactly what a Rust
/// `#[repr(C)]` 16-byte struct does under `extern "C"` on this target.
#[repr(C)] #[derive(Clone, Copy, Debug, PartialEq)]
pub struct fc64_t { pub re: f64, pub im: f64 }

extern "C" {
    // Declarations whose signatures use the types above (symbols exported by libalm.dll, all measured):
    pub fn amd_nexttoward(x: c_double, y: c_long_double) -> c_double;
    pub fn amd_nexttowardf(x: f32, y: c_long_double) -> f32;
    pub fn amd_cexpf(y: fc32_t) -> fc32_t;
    pub fn amd_cexp(x: fc64_t) -> fc64_t;
    pub fn amd_clogf(y: fc32_t) -> fc32_t;
    pub fn amd_clog(x: fc64_t) -> fc64_t;
    pub fn amd_cpowf(x: fc32_t, y: fc32_t) -> fc32_t;
    pub fn amd_cpow(x: fc64_t, y: fc64_t) -> fc64_t;
}

// ---- RNG -------------------------------------------------------------------------------------
/// Width MEASURED earlier (RNG-WIDTH-MEASUREMENT.md): 4 bytes in the LP64 DLL, 8 in the ILP64 DLL.
#[cfg(not(feature = "ilp64"))] pub type rng_int_t = c_int;
#[cfg(feature = "ilp64")]      pub type rng_int_t = i64;

/// User generator INITIALISER callback (same shape for the S and D families; MEASURED: 7 args).
/// Meaning of the 7 pointers as MEASURED by pointer identity / dereference:
///   0 genid*  (pointer to a DLL-local copy of genid)      1 subid* (adjacent DLL-local copy)
///   2 seed*   (== caller's seed array)                    3 lseed* (== caller's lseed)
///   4 state*  (== caller's state array, start of it)      5 lstate* (DLL-LOCAL value = caller lstate - 3, NOT the caller's lstate)
///   6 info*   (== caller's info)
pub type RNG_DRANDINITIALIZEUSER_UINI = Option<unsafe extern "C" fn(*mut rng_int_t, *mut rng_int_t, *mut rng_int_t, *mut rng_int_t, *mut rng_int_t, *mut rng_int_t, *mut rng_int_t)>;
pub type RNG_SRANDINITIALIZEUSER_UINI = Option<unsafe extern "C" fn(*mut rng_int_t, *mut rng_int_t, *mut rng_int_t, *mut rng_int_t, *mut rng_int_t, *mut rng_int_t, *mut rng_int_t)>;

/// User generator callback (guide-stated D shape, CONFIRMED by measurement): 4 args.
///   0 n* (rng_int_t, count)   1 state* (== caller's state)   2 r* (double, n elements)   3 info* (== caller's info)
pub type RNG_DRANDINITIALIZEUSER_UGEN = Option<unsafe extern "C" fn(*mut rng_int_t, *mut rng_int_t, *mut f64, *mut rng_int_t)>;
/// S shape (the guide does NOT state it; MEASURED): identical to D but the output array is `float`.
pub type RNG_SRANDINITIALIZEUSER_UGEN = Option<unsafe extern "C" fn(*mut rng_int_t, *mut rng_int_t, *mut f32, *mut rng_int_t)>;

// UNMEASURED / DO NOT DEFINE (see TYPES-MEASUREMENT.md section 3):
//  * whether the callbacks receive more than the arguments above (the slots past arg3 / arg6 hold stack residue
//    that cannot be told apart from a real extra parameter by this method; the guide's 4-arg / 7-arg typedefs are used);
//  * the DLL's own defective user-state bookkeeping on 64-bit Windows (state[hi32+2] write, -4 on generate): the
//    `(s|d)randinitializeuser` entry points are NOT declared here, because a safe, working call sequence was not found.
