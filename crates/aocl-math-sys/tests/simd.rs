//! NIGHTLY ONLY (feature `simd_ffi`): by-value SIMD functions give the same lane values as the scalar functions.
//! Run: `cargo +nightly test -p aocl-math-sys --features simd_ffi --test simd`. Lanes carry distinct inputs so a lane
//! swap or a wrong by-value convention (garbage / different lane order) shows; control: the cos function must differ.
#![cfg(feature = "simd_ffi")]
#![feature(simd_ffi)]
use aocl_math_sys as s;
use core::arch::x86_64::*;
use core::mem::transmute;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 4e-6 * b.abs().max(1e-3)
}

#[test]
fn sse_float_and_double() {
    let xf = [0.1f32, 0.7, 1.3, 2.9];
    let xd = [0.2f64, 1.1];
    unsafe {
        let r: [f32; 4] = transmute(s::amd_vrs4_sinf(transmute(xf)));
        let c: [f32; 4] = transmute(s::amd_vrs4_cosf(transmute(xf)));
        for i in 0..4 {
            assert!(
                close(r[i] as f64, s::amd_sinf(xf[i]) as f64),
                "sinf lane {i}: {r:?}"
            );
            assert!(
                !close(c[i] as f64, r[i] as f64),
                "control: cos == sin at lane {i}"
            );
        }
        let d: [f64; 2] = transmute(s::amd_vrd2_sin(transmute(xd)));
        for i in 0..2 {
            assert!(close(d[i], s::amd_sin(xd[i])), "sin lane {i}: {d:?}");
        }
    }
}

#[test]
fn avx_float_and_double() {
    if !is_x86_feature_detected!("avx") {
        return;
    }
    unsafe { avx_impl() }
}

#[target_feature(enable = "avx")]
unsafe fn avx_impl() {
    let xf = [0.1f32, 0.3, 0.7, 1.0, 1.3, 1.7, 2.2, 2.9];
    let xd = [0.2f64, 0.9, 1.4, 2.5];
    unsafe {
        let r: [f32; 8] = transmute(s::amd_vrs8_sinf(transmute(xf)));
        for i in 0..8 {
            assert!(
                close(r[i] as f64, s::amd_sinf(xf[i]) as f64),
                "sinf lane {i}: {r:?}"
            );
        }
        let d: [f64; 4] = transmute(s::amd_vrd4_sin(transmute(xd)));
        for i in 0..4 {
            assert!(close(d[i], s::amd_sin(xd[i])), "sin lane {i}: {d:?}");
        }
    }
}

#[test]
fn avx512_float() {
    if !is_x86_feature_detected!("avx512f") {
        return;
    }
    unsafe { avx512_impl() }
}

#[target_feature(enable = "avx512f")]
unsafe fn avx512_impl() {
    let xf: [f32; 16] = core::array::from_fn(|i| 0.1 + 0.17 * i as f32);
    unsafe {
        let r: [f32; 16] = transmute(s::amd_vrs16_sinf(transmute(xf)));
        for i in 0..16 {
            assert!(
                close(r[i] as f64, s::amd_sinf(xf[i]) as f64),
                "sinf lane {i}: {r:?}"
            );
        }
    }
}

#[test]
fn all_154_simd_symbols_link() {
    // Taking the address forces the linker to resolve every declared symbol against the installed libalm.
    let addrs: Vec<usize> = vec![
        s::amd_vrs4_sinf as usize,
        s::amd_vrd2_sin as usize,
        s::amd_vrs4_cosf as usize,
        s::amd_vrd2_cos as usize,
        s::amd_vrs4_tanf as usize,
        s::amd_vrd2_tan as usize,
        s::amd_vrs4_sincosf as usize,
        s::amd_vrs4_asinf as usize,
        s::amd_vrd2_asin as usize,
        s::amd_vrs4_acosf as usize,
        s::amd_vrd2_acos as usize,
        s::amd_vrs4_atanf as usize,
        s::amd_vrd2_atan as usize,
        s::amd_vrd2_atan2 as usize,
        s::amd_vrs4_coshf as usize,
        s::amd_vrd2_cosh as usize,
        s::amd_vrs4_tanhf as usize,
        s::amd_vrd2_tanh as usize,
        s::amd_vrs4_expf as usize,
        s::amd_vrd2_exp as usize,
        s::amd_vrs4_exp2f as usize,
        s::amd_vrd2_exp2 as usize,
        s::amd_vrs4_exp10f as usize,
        s::amd_vrd2_exp10 as usize,
        s::amd_vrs4_expm1f as usize,
        s::amd_vrs4_logf as usize,
        s::amd_vrd2_log as usize,
        s::amd_vrs4_log2f as usize,
        s::amd_vrd2_log2 as usize,
        s::amd_vrs4_log10f as usize,
        s::amd_vrd2_log10 as usize,
        s::amd_vrs4_log1pf as usize,
        s::amd_vrd2_log1p as usize,
        s::amd_vrs4_powf as usize,
        s::amd_vrd2_pow as usize,
        s::amd_vrs4_powxf as usize,
        s::amd_vrd2_powx as usize,
        s::amd_vrs4_sqrtf as usize,
        s::amd_vrd2_sqrt as usize,
        s::amd_vrs4_cbrtf as usize,
        s::amd_vrd2_cbrt as usize,
        s::amd_vrs4_erff as usize,
        s::amd_vrd2_erf as usize,
        s::amd_vrs4_erfcf as usize,
        s::amd_vrd2_erfc as usize,
        s::amd_vrd2_erfinv as usize,
        s::amd_vrd2_erfcinv as usize,
        s::amd_vrs4_roundf as usize,
        s::amd_vrd2_round as usize,
        s::amd_vrs4_fabsf as usize,
        s::amd_vrd2_fabs as usize,
        s::amd_vrs8_sinf as usize,
        s::amd_vrd4_sin as usize,
        s::amd_vrs8_cosf as usize,
        s::amd_vrd4_cos as usize,
        s::amd_vrs8_tanf as usize,
        s::amd_vrd4_tan as usize,
        s::amd_vrs8_sincosf as usize,
        s::amd_vrd4_sincos as usize,
        s::amd_vrs8_asinf as usize,
        s::amd_vrd4_asin as usize,
        s::amd_vrs8_acosf as usize,
        s::amd_vrd4_acos as usize,
        s::amd_vrs8_atanf as usize,
        s::amd_vrd4_atan as usize,
        s::amd_vrd4_atan2 as usize,
        s::amd_vrs8_coshf as usize,
        s::amd_vrd4_cosh as usize,
        s::amd_vrs8_tanhf as usize,
        s::amd_vrd4_tanh as usize,
        s::amd_vrs8_expf as usize,
        s::amd_vrd4_exp as usize,
        s::amd_vrs8_exp2f as usize,
        s::amd_vrd4_exp2 as usize,
        s::amd_vrs8_exp10f as usize,
        s::amd_vrd4_exp10 as usize,
        s::amd_vrs8_logf as usize,
        s::amd_vrd4_log as usize,
        s::amd_vrs8_log2f as usize,
        s::amd_vrd4_log2 as usize,
        s::amd_vrs8_log10f as usize,
        s::amd_vrs8_powf as usize,
        s::amd_vrd4_pow as usize,
        s::amd_vrs8_powxf as usize,
        s::amd_vrd4_powx as usize,
        s::amd_vrs8_sqrtf as usize,
        s::amd_vrd4_sqrt as usize,
        s::amd_vrs8_cbrtf as usize,
        s::amd_vrs8_erff as usize,
        s::amd_vrd4_erf as usize,
        s::amd_vrs8_erfcf as usize,
        s::amd_vrd4_erfc as usize,
        s::amd_vrd4_erfinv as usize,
        s::amd_vrd4_erfcinv as usize,
        s::amd_vrs8_roundf as usize,
        s::amd_vrd4_round as usize,
        s::amd_vrs8_fabsf as usize,
        s::amd_vrd4_fabs as usize,
        s::amd_vrs16_sinf as usize,
        s::amd_vrd8_sin as usize,
        s::amd_vrs16_cosf as usize,
        s::amd_vrd8_cos as usize,
        s::amd_vrs16_tanf as usize,
        s::amd_vrd8_tan as usize,
        s::amd_vrs16_sincosf as usize,
        s::amd_vrd8_sincos as usize,
        s::amd_vrs16_asinf as usize,
        s::amd_vrd8_asin as usize,
        s::amd_vrs16_acosf as usize,
        s::amd_vrd8_acos as usize,
        s::amd_vrs16_atanf as usize,
        s::amd_vrd8_atan as usize,
        s::amd_vrd8_atan2 as usize,
        s::amd_vrs16_coshf as usize,
        s::amd_vrd8_cosh as usize,
        s::amd_vrs16_tanhf as usize,
        s::amd_vrd8_tanh as usize,
        s::amd_vrs16_expf as usize,
        s::amd_vrd8_exp as usize,
        s::amd_vrs16_exp2f as usize,
        s::amd_vrd8_exp2 as usize,
        s::amd_vrs16_exp10f as usize,
        s::amd_vrd8_exp10 as usize,
        s::amd_vrs16_logf as usize,
        s::amd_vrd8_log as usize,
        s::amd_vrs16_log2f as usize,
        s::amd_vrd8_log2 as usize,
        s::amd_vrs16_log10f as usize,
        s::amd_vrs16_powf as usize,
        s::amd_vrd8_pow as usize,
        s::amd_vrs16_powxf as usize,
        s::amd_vrd8_powx as usize,
        s::amd_vrs16_sqrtf as usize,
        s::amd_vrd8_sqrt as usize,
        s::amd_vrs16_erff as usize,
        s::amd_vrd8_erf as usize,
        s::amd_vrs16_erfcf as usize,
        s::amd_vrd8_erfc as usize,
        s::amd_vrd8_erfinv as usize,
        s::amd_vrd8_erfcinv as usize,
        s::amd_vrs16_roundf as usize,
        s::amd_vrd8_round as usize,
        s::amd_vrd2_cdfnorm as usize,
        s::amd_vrd4_cdfnorm as usize,
        s::amd_vrd8_cdfnorm as usize,
        s::amd_vrd2_cdfnorminv as usize,
        s::amd_vrd4_cdfnorminv as usize,
        s::amd_vrd8_cdfnorminv as usize,
        s::amd_vrd2_linearfrac as usize,
        s::amd_vrs4_linearfracf as usize,
        s::amd_vrd4_linearfrac as usize,
        s::amd_vrs8_linearfracf as usize,
        s::amd_vrd8_linearfrac as usize,
        s::amd_vrs16_linearfracf as usize,
    ];
    assert_eq!(addrs.len(), 154);
    assert!(addrs.iter().all(|a| *a != 0));
}
