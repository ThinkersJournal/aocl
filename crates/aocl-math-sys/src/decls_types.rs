// The 8 LibM functions whose guide prototypes use `long double`, `fc32_t`, `fc64_t` (types the guide never defines).
// The ABI was MEASURED on the installed AOCL 6.0.68 libalm: docs/cleanroom/types/TYPES-MEASUREMENT.md. Windows x86_64 ONLY
// (compiled only on Windows): Linux was not measured (there `long double` is 80-bit and the complex ABI differs).
use core::ffi::{c_double, c_float};

/// C `long double` here: MEASURED to be a plain 64-bit IEEE double passed in xmm1 (not 80-bit) on Windows x64.
pub type c_long_double = c_double;

/// MEASURED: `{ re: f32, im: f32 }` (8 bytes), passed in one integer register and returned in rax.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct fc32_t {
    pub re: f32,
    pub im: f32,
}

/// MEASURED: `{ re: f64, im: f64 }` (16 bytes), passed by hidden reference and returned through a hidden pointer.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct fc64_t {
    pub re: f64,
    pub im: f64,
}

extern "C" {
    pub fn amd_nexttoward(x: c_double, y: c_long_double) -> c_double;
    pub fn amd_nexttowardf(x: c_float, y: c_long_double) -> c_float;
    pub fn amd_cexpf(y: fc32_t) -> fc32_t;
    pub fn amd_cexp(x: fc64_t) -> fc64_t;
    pub fn amd_clogf(y: fc32_t) -> fc32_t;
    pub fn amd_clog(x: fc64_t) -> fc64_t;
    pub fn amd_cpowf(x: fc32_t, y: fc32_t) -> fc32_t;
    pub fn amd_cpow(x: fc64_t, y: fc64_t) -> fc64_t;
}
