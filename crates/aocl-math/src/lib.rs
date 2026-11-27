//! Safe Rust wrappers for AOCL-LibM — comprehensive scalar math, AMD-tuned.
//!
//! Covers the entire AOCL-LibM scalar API: trig + inverse trig + atan2;
//! hyperbolic + inverse hyperbolic; exp / log family (exp, exp2, exp10,
//! expm1, log, log2, log10, log1p, logb, ilogb); power family (pow, sqrt,
//! cbrt, hypot); floor/ceil/round/trunc/rint/nearbyint and their integer
//! variants (lrint, llrint, lround, llround); fmod / remainder / remquo;
//! frexp / ldexp / scalbn / scalbln / modf; nextafter / copysign /
//! fdim / fmin / fmax; finite check; erf.
//!
//! Removed in the clean-room rewrite (the AOCL API Guide does not define the types their C signatures use, so the ABI is not
//! derivable): `nexttoward`, `nexttowardf` (`long double`) and the complex `cexp`, `cexpf`, `cpow`, `cpowf`, `cln`, `clnf`
//! (`fc32_t`/`fc64_t`).
//!
//! Each `f32` entry point forwards to the corresponding `amd_*f` symbol.

#![warn(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use aocl_error::{Error, Result};
use aocl_math_sys as sys;
pub use aocl_types::{Complex32, Complex64};

use std::ffi::CString;

// All amd_* scalar entry points are pure functions of their arguments
// (the few with out-pointers — sincos, modf, frexp, remquo — read/write
// only what they're explicitly handed). We mark wrappers `#[inline]` so
// they compile to a direct call.

// =========================================================================
//   Trigonometric
// =========================================================================

/// Cosine, `x` in radians.
#[inline]
pub fn cos(x: f64) -> f64 {
    unsafe { sys::amd_cos(x) }
}
/// Cosine in single precision.
#[inline]
pub fn cosf(x: f32) -> f32 {
    unsafe { sys::amd_cosf(x) }
}

/// Sine, `x` in radians.
#[inline]
pub fn sin(x: f64) -> f64 {
    unsafe { sys::amd_sin(x) }
}
/// Sine in single precision.
#[inline]
pub fn sinf(x: f32) -> f32 {
    unsafe { sys::amd_sinf(x) }
}

/// Tangent, `x` in radians.
#[inline]
pub fn tan(x: f64) -> f64 {
    unsafe { sys::amd_tan(x) }
}
/// Tangent in single precision.
#[inline]
pub fn tanf(x: f32) -> f32 {
    unsafe { sys::amd_tanf(x) }
}

/// Arc cosine.
#[inline]
pub fn acos(x: f64) -> f64 {
    unsafe { sys::amd_acos(x) }
}
#[inline]
pub fn acosf(x: f32) -> f32 {
    unsafe { sys::amd_acosf(x) }
}

/// Arc sine.
#[inline]
pub fn asin(x: f64) -> f64 {
    unsafe { sys::amd_asin(x) }
}
#[inline]
pub fn asinf(x: f32) -> f32 {
    unsafe { sys::amd_asinf(x) }
}

/// Arc tangent.
#[inline]
pub fn atan(x: f64) -> f64 {
    unsafe { sys::amd_atan(x) }
}
#[inline]
pub fn atanf(x: f32) -> f32 {
    unsafe { sys::amd_atanf(x) }
}

/// Two-argument arc tangent.
#[inline]
pub fn atan2(y: f64, x: f64) -> f64 {
    unsafe { sys::amd_atan2(y, x) }
}
#[inline]
pub fn atan2f(y: f32, x: f32) -> f32 {
    unsafe { sys::amd_atan2f(y, x) }
}

/// Compute sin and cos simultaneously.
#[inline]
pub fn sincos(x: f64) -> (f64, f64) {
    let mut s = 0.0_f64;
    let mut c = 0.0_f64;
    unsafe { sys::amd_sincos(x, &mut s, &mut c) };
    (s, c)
}
#[inline]
pub fn sincosf(x: f32) -> (f32, f32) {
    let mut s = 0.0_f32;
    let mut c = 0.0_f32;
    unsafe { sys::amd_sincosf(x, &mut s, &mut c) };
    (s, c)
}

// =========================================================================
//   Hyperbolic
// =========================================================================

#[inline]
pub fn cosh(x: f64) -> f64 {
    unsafe { sys::amd_cosh(x) }
}
#[inline]
pub fn coshf(x: f32) -> f32 {
    unsafe { sys::amd_coshf(x) }
}
#[inline]
pub fn sinh(x: f64) -> f64 {
    unsafe { sys::amd_sinh(x) }
}
#[inline]
pub fn sinhf(x: f32) -> f32 {
    unsafe { sys::amd_sinhf(x) }
}
#[inline]
pub fn tanh(x: f64) -> f64 {
    unsafe { sys::amd_tanh(x) }
}
#[inline]
pub fn tanhf(x: f32) -> f32 {
    unsafe { sys::amd_tanhf(x) }
}
#[inline]
pub fn acosh(x: f64) -> f64 {
    unsafe { sys::amd_acosh(x) }
}
#[inline]
pub fn acoshf(x: f32) -> f32 {
    unsafe { sys::amd_acoshf(x) }
}
#[inline]
pub fn asinh(x: f64) -> f64 {
    unsafe { sys::amd_asinh(x) }
}
#[inline]
pub fn asinhf(x: f32) -> f32 {
    unsafe { sys::amd_asinhf(x) }
}
#[inline]
pub fn atanh(x: f64) -> f64 {
    unsafe { sys::amd_atanh(x) }
}
#[inline]
pub fn atanhf(x: f32) -> f32 {
    unsafe { sys::amd_atanhf(x) }
}

// =========================================================================
//   Exponential / logarithm
// =========================================================================

/// `eˣ`.
#[inline]
pub fn exp(x: f64) -> f64 {
    unsafe { sys::amd_exp(x) }
}
#[inline]
pub fn expf(x: f32) -> f32 {
    unsafe { sys::amd_expf(x) }
}
/// `2ˣ`.
#[inline]
pub fn exp2(x: f64) -> f64 {
    unsafe { sys::amd_exp2(x) }
}
#[inline]
pub fn exp2f(x: f32) -> f32 {
    unsafe { sys::amd_exp2f(x) }
}
/// `10ˣ`.
#[inline]
pub fn exp10(x: f64) -> f64 {
    unsafe { sys::amd_exp10(x) }
}
#[inline]
pub fn exp10f(x: f32) -> f32 {
    unsafe { sys::amd_exp10f(x) }
}
/// `eˣ - 1` (accurate for small x).
#[inline]
pub fn expm1(x: f64) -> f64 {
    unsafe { sys::amd_expm1(x) }
}
#[inline]
pub fn expm1f(x: f32) -> f32 {
    unsafe { sys::amd_expm1f(x) }
}

/// Natural logarithm.
#[inline]
pub fn ln(x: f64) -> f64 {
    unsafe { sys::amd_log(x) }
}
#[inline]
pub fn lnf(x: f32) -> f32 {
    unsafe { sys::amd_logf(x) }
}
/// Log base 2.
#[inline]
pub fn log2(x: f64) -> f64 {
    unsafe { sys::amd_log2(x) }
}
#[inline]
pub fn log2f(x: f32) -> f32 {
    unsafe { sys::amd_log2f(x) }
}
/// Log base 10.
#[inline]
pub fn log10(x: f64) -> f64 {
    unsafe { sys::amd_log10(x) }
}
#[inline]
pub fn log10f(x: f32) -> f32 {
    unsafe { sys::amd_log10f(x) }
}
/// `ln(1 + x)` (accurate for small x).
#[inline]
pub fn ln_1p(x: f64) -> f64 {
    unsafe { sys::amd_log1p(x) }
}
#[inline]
pub fn ln_1pf(x: f32) -> f32 {
    unsafe { sys::amd_log1pf(x) }
}
/// Unbiased exponent of `x`.
#[inline]
pub fn logb(x: f64) -> f64 {
    unsafe { sys::amd_logb(x) }
}
#[inline]
pub fn logbf(x: f32) -> f32 {
    unsafe { sys::amd_logbf(x) }
}
/// Unbiased exponent of `x` as an integer.
#[inline]
pub fn ilogb(x: f64) -> i32 {
    unsafe { sys::amd_ilogb(x) }
}
#[inline]
pub fn ilogbf(x: f32) -> i32 {
    unsafe { sys::amd_ilogbf(x) }
}

// =========================================================================
//   Power / root
// =========================================================================

/// `x^y`.
#[inline]
pub fn pow(x: f64, y: f64) -> f64 {
    unsafe { sys::amd_pow(x, y) }
}
#[inline]
pub fn powf(x: f32, y: f32) -> f32 {
    unsafe { sys::amd_powf(x, y) }
}
/// √x.
#[inline]
pub fn sqrt(x: f64) -> f64 {
    unsafe { sys::amd_sqrt(x) }
}
#[inline]
pub fn sqrtf(x: f32) -> f32 {
    unsafe { sys::amd_sqrtf(x) }
}
/// ∛x.
#[inline]
pub fn cbrt(x: f64) -> f64 {
    unsafe { sys::amd_cbrt(x) }
}
#[inline]
pub fn cbrtf(x: f32) -> f32 {
    unsafe { sys::amd_cbrtf(x) }
}
/// √(x² + y²) without overflow.
#[inline]
pub fn hypot(x: f64, y: f64) -> f64 {
    unsafe { sys::amd_hypot(x, y) }
}
#[inline]
pub fn hypotf(x: f32, y: f32) -> f32 {
    unsafe { sys::amd_hypotf(x, y) }
}

// =========================================================================
//   Absolute value, rounding, integer conversions
// =========================================================================

#[inline]
pub fn fabs(x: f64) -> f64 {
    unsafe { sys::amd_fabs(x) }
}
#[inline]
pub fn fabsf(x: f32) -> f32 {
    unsafe { sys::amd_fabsf(x) }
}
#[inline]
pub fn ceil(x: f64) -> f64 {
    unsafe { sys::amd_ceil(x) }
}
#[inline]
pub fn ceilf(x: f32) -> f32 {
    unsafe { sys::amd_ceilf(x) }
}
#[inline]
pub fn floor(x: f64) -> f64 {
    unsafe { sys::amd_floor(x) }
}
#[inline]
pub fn floorf(x: f32) -> f32 {
    unsafe { sys::amd_floorf(x) }
}
#[inline]
pub fn trunc(x: f64) -> f64 {
    unsafe { sys::amd_trunc(x) }
}
#[inline]
pub fn truncf(x: f32) -> f32 {
    unsafe { sys::amd_truncf(x) }
}
/// Round to nearest integer using current rounding mode (no inexact flag).
#[inline]
pub fn nearbyint(x: f64) -> f64 {
    unsafe { sys::amd_nearbyint(x) }
}
#[inline]
pub fn nearbyintf(x: f32) -> f32 {
    unsafe { sys::amd_nearbyintf(x) }
}
/// Round to nearest integer using current rounding mode (may set inexact).
#[inline]
pub fn rint(x: f64) -> f64 {
    unsafe { sys::amd_rint(x) }
}
#[inline]
pub fn rintf(x: f32) -> f32 {
    unsafe { sys::amd_rintf(x) }
}
/// Round to nearest integer (ties away from zero).
#[inline]
pub fn round(x: f64) -> f64 {
    unsafe { sys::amd_round(x) }
}
#[inline]
pub fn roundf(x: f32) -> f32 {
    unsafe { sys::amd_roundf(x) }
}

/// Round to nearest `i64`-sized integer.
#[inline]
pub fn lrint(x: f64) -> i64 {
    unsafe { sys::amd_lrint(x) as i64 }
}
#[inline]
pub fn lrintf(x: f32) -> i64 {
    unsafe { sys::amd_lrintf(x) as i64 }
}
#[inline]
pub fn llrint(x: f64) -> i64 {
    unsafe { sys::amd_llrint(x) as i64 }
}
#[inline]
pub fn llrintf(x: f32) -> i64 {
    unsafe { sys::amd_llrintf(x) as i64 }
}
#[inline]
pub fn lround(x: f64) -> i64 {
    unsafe { sys::amd_lround(x) as i64 }
}
#[inline]
pub fn lroundf(x: f32) -> i64 {
    unsafe { sys::amd_lroundf(x) as i64 }
}
#[inline]
pub fn llround(x: f64) -> i64 {
    unsafe { sys::amd_llround(x) as i64 }
}
#[inline]
pub fn llroundf(x: f32) -> i64 {
    unsafe { sys::amd_llroundf(x) as i64 }
}

// =========================================================================
//   Modulo / remainder
// =========================================================================

/// `x mod y` with sign matching `x`.
#[inline]
pub fn fmod(x: f64, y: f64) -> f64 {
    unsafe { sys::amd_fmod(x, y) }
}
#[inline]
pub fn fmodf(x: f32, y: f32) -> f32 {
    unsafe { sys::amd_fmodf(x, y) }
}
/// IEEE remainder.
#[inline]
pub fn remainder(x: f64, y: f64) -> f64 {
    unsafe { sys::amd_remainder(x, y) }
}
#[inline]
pub fn remainderf(x: f32, y: f32) -> f32 {
    unsafe { sys::amd_remainderf(x, y) }
}
/// IEEE remainder + low bits of quotient.
#[inline]
pub fn remquo(x: f64, y: f64) -> (f64, i32) {
    let mut q: i32 = 0;
    let r = unsafe { sys::amd_remquo(x, y, &mut q) };
    (r, q)
}
#[inline]
pub fn remquof(x: f32, y: f32) -> (f32, i32) {
    let mut q: i32 = 0;
    let r = unsafe { sys::amd_remquof(x, y, &mut q) };
    (r, q)
}

// =========================================================================
//   Decompose / scale
// =========================================================================

/// Split `x` into integer and fractional parts. Returns `(fractional, integer)`.
#[inline]
pub fn modf(x: f64) -> (f64, f64) {
    let mut i = 0.0_f64;
    let frac = unsafe { sys::amd_modf(x, &mut i) };
    (frac, i)
}
#[inline]
pub fn modff(x: f32) -> (f32, f32) {
    let mut i = 0.0_f32;
    let frac = unsafe { sys::amd_modff(x, &mut i) };
    (frac, i)
}

/// Decompose `x = m · 2ᵉ` with `m ∈ [0.5, 1.0)` (or 0). Returns `(m, e)`.
#[inline]
pub fn frexp(x: f64) -> (f64, i32) {
    let mut e: i32 = 0;
    let m = unsafe { sys::amd_frexp(x, &mut e) };
    (m, e)
}
#[inline]
pub fn frexpf(x: f32) -> (f32, i32) {
    let mut e: i32 = 0;
    let m = unsafe { sys::amd_frexpf(x, &mut e) };
    (m, e)
}

/// `x · 2ⁿ`.
#[inline]
pub fn ldexp(x: f64, n: i32) -> f64 {
    unsafe { sys::amd_ldexp(x, n) }
}
#[inline]
pub fn ldexpf(x: f32, n: i32) -> f32 {
    unsafe { sys::amd_ldexpf(x, n) }
}
#[inline]
pub fn scalbn(x: f64, n: i32) -> f64 {
    unsafe { sys::amd_scalbn(x, n) }
}
#[inline]
pub fn scalbnf(x: f32, n: i32) -> f32 {
    unsafe { sys::amd_scalbnf(x, n) }
}
#[inline]
pub fn scalbln(x: f64, n: i64) -> f64 {
    unsafe { sys::amd_scalbln(x, n as std::os::raw::c_long) }
}
#[inline]
pub fn scalblnf(x: f32, n: i64) -> f32 {
    unsafe { sys::amd_scalblnf(x, n as std::os::raw::c_long) }
}

// =========================================================================
//   Float ops (sign manipulation, comparison, etc.)
// =========================================================================

#[inline]
pub fn copysign(magnitude: f64, sign: f64) -> f64 {
    unsafe { sys::amd_copysign(magnitude, sign) }
}
#[inline]
pub fn copysignf(magnitude: f32, sign: f32) -> f32 {
    unsafe { sys::amd_copysignf(magnitude, sign) }
}

/// Next representable value after `x` toward `y`.
#[inline]
pub fn nextafter(x: f64, y: f64) -> f64 {
    unsafe { sys::amd_nextafter(x, y) }
}
#[inline]
pub fn nextafterf(x: f32, y: f32) -> f32 {
    unsafe { sys::amd_nextafterf(x, y) }
}
/// `max(x − y, 0)`.
#[inline]
pub fn fdim(x: f64, y: f64) -> f64 {
    unsafe { sys::amd_fdim(x, y) }
}
#[inline]
pub fn fdimf(x: f32, y: f32) -> f32 {
    unsafe { sys::amd_fdimf(x, y) }
}
#[inline]
pub fn fmax(x: f64, y: f64) -> f64 {
    unsafe { sys::amd_fmax(x, y) }
}
#[inline]
pub fn fmaxf(x: f32, y: f32) -> f32 {
    unsafe { sys::amd_fmaxf(x, y) }
}
#[inline]
pub fn fmin(x: f64, y: f64) -> f64 {
    unsafe { sys::amd_fmin(x, y) }
}
#[inline]
pub fn fminf(x: f32, y: f32) -> f32 {
    unsafe { sys::amd_fminf(x, y) }
}

/// Returns `true` if `x` is finite (neither infinite nor NaN).
#[inline]
pub fn finite(x: f64) -> bool {
    unsafe { sys::amd_finite(x) != 0 }
}
#[inline]
pub fn finitef(x: f32) -> bool {
    unsafe { sys::amd_finitef(x) != 0 }
}

/// Construct a NaN value, optionally with a payload tag.
pub fn nan(tag: &str) -> Result<f64> {
    let cs = CString::new(tag)
        .map_err(|e| Error::InvalidArgument(format!("nan: tag contained interior NUL: {e}")))?;
    Ok(unsafe { sys::amd_nan(cs.as_ptr()) })
}
pub fn nanf(tag: &str) -> Result<f32> {
    let cs = CString::new(tag)
        .map_err(|e| Error::InvalidArgument(format!("nanf: tag contained interior NUL: {e}")))?;
    Ok(unsafe { sys::amd_nanf(cs.as_ptr()) })
}

/// Error function `erf(x) = (2/√π) ∫₀ˣ e^(-t²) dt`.
#[inline]
pub fn erf(x: f64) -> f64 {
    unsafe { sys::amd_erf(x) }
}
#[inline]
pub fn erff(x: f32) -> f32 {
    unsafe { sys::amd_erff(x) }
}

// =========================================================================
//   Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, eps: f64) {
        assert!((a - b).abs() < eps, "|{a} - {b}| > {eps}");
    }

    #[test]
    fn trig_round_trips() {
        approx(asin(sin(0.7)), 0.7, 1e-12);
        approx(acos(cos(0.7)), 0.7, 1e-12);
        approx(atan(tan(0.5)), 0.5, 1e-12);
        approx(atan2(1.0, 1.0), std::f64::consts::FRAC_PI_4, 1e-12);
        let (s, c) = sincos(0.7);
        approx(s * s + c * c, 1.0, 1e-12);
    }

    #[test]
    fn hyperbolic_consistency() {
        let x = 0.5_f64;
        approx(asinh(sinh(x)), x, 1e-12);
        approx(acosh(cosh(x)), x, 1e-12);
        approx(atanh(tanh(x)), x, 1e-12);
        approx(cosh(x) * cosh(x) - sinh(x) * sinh(x), 1.0, 1e-12);
    }

    #[test]
    fn exp_log_family() {
        approx(exp(1.0), std::f64::consts::E, 1e-12);
        approx(exp2(10.0), 1024.0, 1e-12);
        approx(exp10(3.0), 1000.0, 1e-12);
        approx(expm1(0.0), 0.0, 1e-12);
        approx(ln(std::f64::consts::E), 1.0, 1e-12);
        approx(log2(1024.0), 10.0, 1e-12);
        approx(log10(1000.0), 3.0, 1e-12);
        approx(ln_1p(0.0), 0.0, 1e-12);
        approx(logb(8.0), 3.0, 1e-12);
        assert_eq!(ilogb(8.0), 3);
    }

    #[test]
    fn power_root_family() {
        approx(pow(2.0, 10.0), 1024.0, 1e-12);
        approx(sqrt(2.0), std::f64::consts::SQRT_2, 1e-12);
        approx(cbrt(27.0), 3.0, 1e-12);
        approx(hypot(3.0, 4.0), 5.0, 1e-12);
    }

    #[test]
    fn rounding_family() {
        approx(fabs(-3.0), 3.0, 1e-12);
        approx(ceil(2.3), 3.0, 1e-12);
        approx(floor(2.7), 2.0, 1e-12);
        approx(trunc(-2.7), -2.0, 1e-12);
        approx(round(2.5), 3.0, 1e-12);
        assert_eq!(lround(2.7), 3);
        assert_eq!(llround(-2.7), -3);
    }

    #[test]
    fn modulo_remainder() {
        approx(fmod(5.0, 3.0), 2.0, 1e-12);
        approx(remainder(5.0, 3.0), -1.0, 1e-12);
        let (r, q) = remquo(5.0, 3.0);
        approx(r, -1.0, 1e-12);
        assert_eq!(q, 2);
    }

    #[test]
    fn frexp_modf() {
        let (m, e) = frexp(8.0);
        approx(m, 0.5, 1e-12);
        assert_eq!(e, 4);
        let (frac, int) = modf(3.7);
        approx(frac, 0.7, 1e-12);
        approx(int, 3.0, 1e-12);
        approx(ldexp(1.0, 10), 1024.0, 1e-12);
        approx(scalbn(1.0, 5), 32.0, 1e-12);
    }

    #[test]
    fn float_ops() {
        approx(copysign(3.0, -1.0), -3.0, 1e-12);
        approx(fdim(5.0, 3.0), 2.0, 1e-12);
        approx(fdim(3.0, 5.0), 0.0, 1e-12);
        approx(fmax(2.0, 3.0), 3.0, 1e-12);
        approx(fmin(2.0, 3.0), 2.0, 1e-12);
        assert!(finite(1.0));
        assert!(!finite(f64::INFINITY));
        assert!(!finite(f64::NAN));
        assert!(nextafter(1.0, 2.0) > 1.0);
    }

    #[test]
    fn erf_known_values() {
        approx(erf(0.0), 0.0, 1e-12);
        // erf(1) ≈ 0.8427007929497149
        approx(erf(1.0), 0.842_700_792_949_715, 1e-12);
    }

    #[test]
    fn nan_with_tag_round_trips() {
        let n = nan("123").unwrap();
        assert!(n.is_nan());
    }

    #[test]
    fn f32_variants_match() {
        assert!((expf(1.0_f32) - std::f32::consts::E).abs() < 1e-5);
        assert!((sqrtf(2.0_f32) - std::f32::consts::SQRT_2).abs() < 1e-5);
        assert!((sinhf(0.5_f32) - 0.5_f32.sinh()).abs() < 1e-5);
    }

    // ---- f32 / rounding-mode / scaling coverage. Expected values come from Rust std or exact identities, never from the library. ----

    /// `got` equals `want` within `rel` relative error (absolute `1e-7` when `want` is ~0); NaN matches NaN, infinities match exactly.
    fn close32(got: f32, want: f32, rel: f32, what: &str) {
        if want.is_nan() {
            assert!(got.is_nan(), "{what}: got {got}, want NaN");
        } else if want.is_infinite() {
            assert_eq!(got, want, "{what}");
        } else {
            let tol = rel * want.abs().max(1e-7 / rel.max(1e-9));
            assert!(
                (got - want).abs() <= tol,
                "{what}: got {got}, want {want} (tol {tol})"
            );
        }
    }

    /// Distinct per-lane inputs through a wrapper vs the std function (lane order mistakes would show).
    macro_rules! unary32 {
        ($name:ident, $f:ident, $std:expr, [$($x:expr),+ $(,)?]) => {
            #[test]
            fn $name() {
                let std_fn: fn(f32) -> f32 = $std;
                for x in [$($x),+] {
                    close32($f(x), std_fn(x), 2e-6, &format!("{}({x})", stringify!($f)));
                }
            }
        };
    }
    unary32!(
        cosf_vs_std,
        cosf,
        f32::cos,
        [0.0, 0.3, 1.0, 2.0, 3.0, -1.7, 10.0]
    );
    unary32!(
        sinf_vs_std,
        sinf,
        f32::sin,
        [0.1, 0.3, 1.0, 2.0, 3.0, -1.7, 10.0]
    );
    unary32!(tanf_vs_std, tanf, f32::tan, [0.1, 0.3, 1.0, -1.2, 2.0, 3.0]);
    unary32!(
        acosf_vs_std,
        acosf,
        f32::acos,
        [-1.0, -0.6, -0.1, 0.0, 0.25, 0.9, 1.0]
    );
    unary32!(
        asinf_vs_std,
        asinf,
        f32::asin,
        [-1.0, -0.6, -0.1, 0.25, 0.5, 0.9, 1.0]
    );
    unary32!(
        atanf_vs_std,
        atanf,
        f32::atan,
        [-100.0, -2.0, -0.4, 0.1, 0.7, 3.0, 1000.0]
    );
    unary32!(
        coshf_vs_std,
        coshf,
        f32::cosh,
        [0.0, 0.2, -0.7, 1.5, 3.0, -5.0]
    );
    unary32!(
        tanhf_vs_std,
        tanhf,
        f32::tanh,
        [-6.0, -1.0, -0.2, 0.1, 0.8, 2.5]
    );
    unary32!(
        asinhf_vs_std,
        asinhf,
        f32::asinh,
        [-50.0, -2.0, -0.3, 0.1, 0.9, 7.0]
    );
    unary32!(
        acoshf_vs_std,
        acoshf,
        f32::acosh,
        [1.0, 1.1, 1.5, 2.0, 10.0, 500.0]
    );
    unary32!(
        atanhf_vs_std,
        atanhf,
        f32::atanh,
        [-0.9, -0.5, -0.1, 0.2, 0.6, 0.95]
    );
    unary32!(
        exp2f_vs_std,
        exp2f,
        f32::exp2,
        [-10.0, -1.5, 0.0, 0.5, 3.0, 20.0]
    );
    unary32!(
        expm1f_vs_std,
        expm1f,
        f32::exp_m1,
        [-3.0, -0.5, 1e-4, 0.3, 1.0, 5.0]
    );
    unary32!(
        lnf_vs_std,
        lnf,
        f32::ln,
        [1e-6, 0.1, 0.5, 1.0, 2.0, 100.0, 1e6]
    );
    unary32!(
        log2f_vs_std,
        log2f,
        f32::log2,
        [1e-6, 0.1, 0.5, 3.0, 8.0, 1000.0]
    );
    unary32!(
        log10f_vs_std,
        log10f,
        f32::log10,
        [1e-6, 0.1, 0.5, 3.0, 100.0, 1e6]
    );
    unary32!(
        ln_1pf_vs_std,
        ln_1pf,
        f32::ln_1p,
        [-0.9, -0.5, 1e-5, 0.3, 1.0, 100.0]
    );
    unary32!(
        cbrtf_vs_std,
        cbrtf,
        f32::cbrt,
        [-1000.0, -8.0, -0.001, 0.5, 2.0, 27.0, 1e9]
    );
    unary32!(
        ceilf_vs_std,
        ceilf,
        f32::ceil,
        [-2.5, -2.0, -0.1, 0.0, 0.1, 2.0, 2.5, 1e7]
    );
    unary32!(
        floorf_vs_std,
        floorf,
        f32::floor,
        [-2.5, -2.0, -0.1, 0.0, 0.1, 2.0, 2.5, 1e7]
    );
    unary32!(
        truncf_vs_std,
        truncf,
        f32::trunc,
        [-2.9, -0.9, 0.0, 0.9, 2.9, 1e7]
    );
    unary32!(
        roundf_vs_std,
        roundf,
        f32::round,
        [-3.5, -2.5, -0.5, 0.4, 0.5, 1.5, 2.5, 3.5]
    );
    unary32!(
        fabsf_vs_std,
        fabsf,
        f32::abs,
        [-0.0, -3.5, -1e-30, 0.0, 2.25, f32::NEG_INFINITY]
    );
    unary32!(
        rintf_ties_even,
        rintf,
        f32::round_ties_even,
        [-3.5, -2.5, -1.5, -0.5, 0.5, 1.5, 2.5, 3.5, 2.4, -2.6]
    );
    unary32!(
        nearbyintf_ties_even,
        nearbyintf,
        f32::round_ties_even,
        [-3.5, -2.5, -1.5, -0.5, 0.5, 1.5, 2.5, 3.5, 2.4, -2.6]
    );

    #[test]
    fn exp10f_integer_powers() {
        // 10^k for small integer k is exactly representable closed-form; tolerance 2e-6 relative.
        for (k, want) in [
            (-3.0_f32, 1e-3_f32),
            (0.0, 1.0),
            (1.0, 10.0),
            (2.0, 100.0),
            (5.0, 1e5),
            (-1.0, 0.1),
            (10.0, 1e10),
        ] {
            close32(exp10f(k), want, 2e-6, &format!("exp10f({k})"));
        }
        close32(exp10f(0.5), 10.0_f32.sqrt(), 2e-6, "exp10f(0.5)");
    }

    #[test]
    fn atan2f_quadrants_vs_std() {
        let ys = [1.0_f32, 1.0, -1.0, -1.0, 0.5, 3.0, -2.0];
        let xs = [1.0_f32, -1.0, -1.0, 1.0, 2.0, -0.25, 0.0];
        for (y, x) in ys.iter().zip(xs.iter()) {
            close32(
                atan2f(*y, *x),
                y.atan2(*x),
                2e-6,
                &format!("atan2f({y},{x})"),
            );
        }
        close32(
            atan2f(1.0, 1.0),
            std::f32::consts::FRAC_PI_4,
            2e-6,
            "atan2f(1,1)",
        );
        close32(
            atan2f(1.0, -1.0),
            3.0 * std::f32::consts::FRAC_PI_4,
            2e-6,
            "atan2f(1,-1)",
        );
    }

    #[test]
    fn sincosf_matches_scalar_forms() {
        let pi = std::f32::consts::PI;
        for x in [0.0_f32, pi / 6.0, 0.7, pi / 2.0, 2.0, -1.3, 5.0] {
            let (s, c) = sincosf(x);
            close32(s, x.sin(), 2e-6, &format!("sincosf({x}).sin"));
            close32(c, x.cos(), 2e-6, &format!("sincosf({x}).cos"));
        }
        let (s, c) = sincosf(pi / 6.0);
        close32(s, 0.5, 2e-6, "sin(pi/6)");
        close32(c, 3.0_f32.sqrt() / 2.0, 2e-6, "cos(pi/6)");
    }

    #[test]
    fn powf_hypotf_vs_std() {
        for (x, y) in [
            (2.0_f32, 10.0_f32),
            (2.0, 0.5),
            (9.0, 0.5),
            (0.5, 3.0),
            (10.0, -2.0),
            (1.5, 2.5),
            (-2.0, 3.0),
            (-2.0, 2.0),
        ] {
            close32(powf(x, y), x.powf(y), 2e-6, &format!("powf({x},{y})"));
        }
        close32(powf(5.0, 0.0), 1.0, 1e-9, "x^0");
        for (x, y) in [
            (3.0_f32, 4.0_f32),
            (5.0, 12.0),
            (-8.0, 15.0),
            (1e-3, 2e-3),
            (1e20, 1e20),
            (0.0, -7.0),
        ] {
            close32(hypotf(x, y), x.hypot(y), 2e-6, &format!("hypotf({x},{y})"));
        }
        close32(hypotf(3.0, 4.0), 5.0, 2e-6, "3-4-5");
        close32(
            hypotf(1e30, 1e30),
            1e30 * std::f32::consts::SQRT_2,
            2e-6,
            "no overflow",
        );
    }

    #[test]
    fn erff_known_values() {
        // Tabulated erf values (Abramowitz & Stegun), independent of the library.
        for (x, want) in [
            (0.0_f32, 0.0_f32),
            (0.5, 0.520_499_9),
            (1.0, 0.842_700_8),
            (2.0, 0.995_322_3),
            (-1.0, -0.842_700_8),
            (6.0, 1.0),
        ] {
            close32(erff(x), want, 2e-6, &format!("erff({x})"));
        }
        assert_eq!(erff(f32::INFINITY), 1.0);
        assert_eq!(erff(f32::NEG_INFINITY), -1.0);
        assert!(erff(f32::NAN).is_nan());
    }

    #[test]
    fn integer_rounding_ties_f64_f32() {
        // (input, rint/lrint/llrint = ties-to-even in the default rounding mode, lround/llround = ties away from zero)
        let cases: [(f64, i64, i64); 10] = [
            (0.5, 0, 1),
            (1.5, 2, 2),
            (2.5, 2, 3),
            (3.5, 4, 4),
            (-0.5, 0, -1),
            (-1.5, -2, -2),
            (-2.5, -2, -3),
            (2.4, 2, 2),
            (-2.6, -3, -3),
            (1.0e9, 1_000_000_000, 1_000_000_000),
        ];
        for (x, even, away) in cases {
            assert_eq!(lrint(x), even, "lrint({x})");
            assert_eq!(llrint(x), even, "llrint({x})");
            assert_eq!(lround(x), away, "lround({x})");
            assert_eq!(llround(x), away, "llround({x})");
            assert_eq!(rint(x), even as f64, "rint({x})");
            assert_eq!(nearbyint(x), even as f64, "nearbyint({x})");
            let f = x as f32;
            assert_eq!(lrintf(f), even, "lrintf({f})");
            assert_eq!(llrintf(f), even, "llrintf({f})");
            assert_eq!(lroundf(f), away, "lroundf({f})");
            assert_eq!(llroundf(f), away, "llroundf({f})");
        }
        // Large values that need 64 bits (above i32::MAX) must not be truncated.
        assert_eq!(llrint(5.0e12), 5_000_000_000_000);
        assert_eq!(llround(-5.0e12), -5_000_000_000_000);
    }

    #[test]
    fn rint_sign_of_zero() {
        assert!(rint(-0.25).is_sign_negative() && rint(-0.25) == 0.0);
        assert!(nearbyint(-0.25).is_sign_negative());
        assert!(rintf(-0.25).is_sign_negative());
        assert!(nearbyintf(-0.4).is_sign_negative());
        assert!(!rintf(0.25).is_sign_negative());
    }

    #[test]
    fn modff_frexpf_ldexpf_scalbf() {
        for x in [3.75_f32, -3.75, 2.5, 0.25, -0.5, 100.125, 7.0, -3.0] {
            let (frac, int) = modff(x);
            assert_eq!(int, x.trunc(), "modff({x}) integer part");
            assert_eq!(frac, x - x.trunc(), "modff({x}) fractional part");
        }
        let (f, i) = modff(-3.0);
        assert!(
            f == 0.0 && f.is_sign_negative() && i == -3.0,
            "modff(-3) = ({f}, {i})"
        );

        // frexpf: x = m * 2^e with |m| in [0.5, 1).
        assert_eq!(frexpf(12.0), (0.75, 4));
        assert_eq!(frexpf(0.0), (0.0, 0));
        assert_eq!(frexpf(1.0), (0.5, 1));
        assert_eq!(frexpf(-0.25), (-0.5, -1));
        for x in [0.1_f32, -7.5, 1e10, 3e-20, 123456.0] {
            let (m, e) = frexpf(x);
            assert!((0.5..1.0).contains(&m.abs()), "frexpf({x}) mantissa {m}");
            assert_eq!(ldexpf(m, e), x, "ldexpf round trip {x}");
            assert_eq!(scalbnf(m, e), x, "scalbnf round trip {x}");
            assert_eq!(scalblnf(m, e as i64), x, "scalblnf round trip {x}");
        }
        assert_eq!(ldexpf(0.75, -2), 0.1875);
        assert_eq!(scalbnf(1.5, 3), 12.0);
        assert_eq!(scalblnf(3.0, 4), 48.0);
        assert_eq!(scalblnf(1.0, -3), 0.125);
        assert_eq!(scalbnf(1.0, 200), f32::INFINITY);
        assert_eq!(scalbnf(-1.0, 200), f32::NEG_INFINITY);
        assert_eq!(scalbnf(1.0, -300), 0.0);
    }

    #[test]
    fn scalbln_f64() {
        assert_eq!(scalbln(3.0, 4), 48.0);
        assert_eq!(scalbln(1.0, -3), 0.125);
        assert_eq!(scalbln(-1.5, 10), -1536.0);
        assert_eq!(scalbln(1.0, 5_000), f64::INFINITY);
        assert_eq!(scalbln(1.0, -5_000), 0.0);
        let (m, e) = frexp(1234.5678);
        assert_eq!(scalbln(m, e as i64), 1234.5678);
    }

    #[test]
    fn logbf_ilogbf_exponents() {
        for (x, e) in [
            (1.0_f32, 0),
            (8.0, 3),
            (0.25, -2),
            (0.1, -4),
            (1000.0, 9),
            (-16.0, 4),
            (1.99, 0),
            (2.0, 1),
        ] {
            assert_eq!(ilogbf(x), e, "ilogbf({x})");
            assert_eq!(logbf(x), e as f32, "logbf({x})");
        }
        assert_eq!(logbf(0.0), f32::NEG_INFINITY);
        assert_eq!(logbf(f32::INFINITY), f32::INFINITY);
        assert!(logbf(f32::NAN).is_nan());
    }

    #[test]
    fn fmodf_remainderf_remquof() {
        for (x, y) in [
            (5.5_f32, 2.0_f32),
            (-5.5, 2.0),
            (5.5, -2.0),
            (-5.5, -2.0),
            (7.0, 7.0),
            (1.0, 3.0),
            (1e6, 7.0),
        ] {
            assert_eq!(fmodf(x, y), x % y, "fmodf({x},{y})");
        }
        assert_eq!(fmodf(-5.5, 2.0), -1.5);
        assert!(fmodf(5.0, 0.0).is_nan());
        assert!(fmodf(f32::INFINITY, 2.0).is_nan());
        assert_eq!(fmodf(3.0, f32::INFINITY), 3.0);
        // IEEE remainder: x - n*y with n = x/y rounded to nearest even.
        for (x, y, want) in [
            (5.0_f32, 3.0_f32, -1.0_f32),
            (7.0, 2.0, -1.0),
            (5.0, 2.0, 1.0),
            (-5.0, 3.0, 1.0),
            (1.0, 3.0, 1.0),
            (10.0, 4.0, 2.0),
        ] {
            assert_eq!(remainderf(x, y), want, "remainderf({x},{y})");
        }
        assert!(remainderf(1.0, 0.0).is_nan());
        let (r, q) = remquof(5.0, 3.0);
        assert_eq!((r, q), (-1.0, 2));
        let (r, q) = remquof(7.0, 2.0); // 3.5 -> quotient rounds to even 4
        assert_eq!(r, -1.0);
        assert_eq!(q & 7, 4);
        let (r, q) = remquof(-7.0, 2.0);
        assert_eq!(r, 1.0);
        assert!(q < 0 && (-q) & 7 == 4, "q = {q}");
        let (r, q) = remquof(9.0, 3.0);
        assert_eq!((r, q), (0.0, 3));
    }

    #[test]
    fn float_ops_f32() {
        // copysignf: magnitude of the first, sign bit of the second (including -0.0).
        assert_eq!(copysignf(3.0, -1.0), -3.0);
        assert_eq!(copysignf(-3.0, 1.0), 3.0);
        assert_eq!(copysignf(-3.0, -0.0), -3.0);
        assert!(copysignf(0.0, -1.0).is_sign_negative());
        assert!(copysignf(f32::NAN, -1.0).is_sign_negative());
        // fdimf = max(x-y, 0)
        assert_eq!(fdimf(5.0, 3.0), 2.0);
        assert_eq!(fdimf(3.0, 5.0), 0.0);
        assert_eq!(fdimf(-1.0, -4.5), 3.5);
        assert!(fdimf(f32::NAN, 1.0).is_nan());
        // fmax/fmin ignore a single NaN
        for (x, y) in [
            (2.0_f32, 3.0_f32),
            (3.0, 2.0),
            (-1.0, -2.0),
            (-5.0, 4.0),
            (1.0, 1.0),
        ] {
            assert_eq!(fmaxf(x, y), x.max(y), "fmaxf({x},{y})");
            assert_eq!(fminf(x, y), x.min(y), "fminf({x},{y})");
        }
        assert_eq!(fmaxf(f32::NAN, 1.0), 1.0);
        assert_eq!(fmaxf(1.0, f32::NAN), 1.0);
        assert_eq!(fminf(f32::NAN, 1.0), 1.0);
        assert_eq!(fminf(1.0, f32::NAN), 1.0);
        // finitef
        assert!(
            finitef(1.0) && finitef(0.0) && finitef(-f32::MAX) && finitef(f32::MIN_POSITIVE / 2.0)
        );
        assert!(!finitef(f32::INFINITY) && !finitef(f32::NEG_INFINITY) && !finitef(f32::NAN));
    }

    #[test]
    fn nextafterf_steps() {
        let up = f32::from_bits(1.0_f32.to_bits() + 1);
        let down = f32::from_bits(1.0_f32.to_bits() - 1);
        assert_eq!(nextafterf(1.0, 2.0), up);
        assert_eq!(nextafterf(1.0, 0.0), down);
        assert_eq!(nextafterf(1.0, 1.0), 1.0);
        assert_eq!(nextafterf(-1.0, -2.0), -up);
        assert_eq!(nextafterf(-1.0, 0.0), -down);
        assert_eq!(nextafterf(0.0, 1.0), f32::from_bits(1)); // smallest subnormal
        assert_eq!(nextafterf(0.0, -1.0), -f32::from_bits(1));
        assert_eq!(nextafterf(f32::MAX, f32::INFINITY), f32::INFINITY);
        assert!(nextafterf(f32::NAN, 1.0).is_nan());
        // f64 form for symmetry of the same identity
        assert_eq!(nextafter(1.0, 2.0), f64::from_bits(1.0_f64.to_bits() + 1));
    }

    #[test]
    fn nanf_is_nan_with_and_without_tag() {
        assert!(nanf("").unwrap().is_nan());
        assert!(nanf("123").unwrap().is_nan());
        assert!(!finitef(nanf("7").unwrap()));
        assert!(nanf("a\0b").is_err());
        assert!(nan("a\0b").is_err());
    }

    #[test]
    fn f32_special_values() {
        assert_eq!(lnf(0.0), f32::NEG_INFINITY);
        assert!(lnf(-1.0).is_nan());
        assert_eq!(lnf(f32::INFINITY), f32::INFINITY);
        assert_eq!(log2f(0.0), f32::NEG_INFINITY);
        assert_eq!(log10f(0.0), f32::NEG_INFINITY);
        assert_eq!(ln_1pf(-1.0), f32::NEG_INFINITY);
        assert_eq!(exp2f(f32::NEG_INFINITY), 0.0);
        assert_eq!(exp2f(200.0), f32::INFINITY);
        assert_eq!(exp10f(100.0), f32::INFINITY);
        assert_eq!(expm1f(f32::NEG_INFINITY), -1.0);
        assert_eq!(atanf(f32::INFINITY), std::f32::consts::FRAC_PI_2);
        assert!(asinf(1.5).is_nan());
        assert!(acosf(-1.5).is_nan());
        assert!(acoshf(0.5).is_nan());
        assert_eq!(atanhf(1.0), f32::INFINITY);
        assert_eq!(tanhf(f32::INFINITY), 1.0);
        assert_eq!(tanhf(f32::NEG_INFINITY), -1.0);
        assert_eq!(coshf(f32::NEG_INFINITY), f32::INFINITY);
        assert!(cosf(f32::INFINITY).is_nan());
        assert!(sinf(f32::INFINITY).is_nan());
        assert!(tanf(f32::INFINITY).is_nan());
        assert!(cbrtf(-27.0) == -3.0);
        assert!(powf(-8.0, 0.5).is_nan());
        assert_eq!(hypotf(f32::INFINITY, f32::NAN), f32::INFINITY);
        assert_eq!(fabsf(-0.0).to_bits(), 0.0_f32.to_bits());
        // exact closed forms
        close32(cosf(0.0), 1.0, 1e-9, "cosf(0)");
        close32(
            asinf(0.5),
            std::f32::consts::FRAC_PI_6,
            2e-6,
            "asin(1/2)=pi/6",
        );
        close32(
            acosf(0.5),
            std::f32::consts::FRAC_PI_3,
            2e-6,
            "acos(1/2)=pi/3",
        );
        close32(tanf(std::f32::consts::FRAC_PI_4), 1.0, 2e-6, "tan(pi/4)");
        close32(acoshf(1.0), 0.0, 2e-6, "acosh(1)");
        close32(lnf(std::f32::consts::E), 1.0, 2e-6, "ln e");
        close32(log2f(1024.0), 10.0, 2e-6, "log2 1024");
        close32(log10f(1000.0), 3.0, 2e-6, "log10 1000");
        close32(cbrtf(27.0), 3.0, 2e-6, "cbrt 27");
        close32(exp2f(10.0), 1024.0, 2e-6, "2^10");
    }
}
