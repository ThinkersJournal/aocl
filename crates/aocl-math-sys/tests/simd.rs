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
            assert!(close(r[i] as f64, s::amd_sinf(xf[i]) as f64), "sinf lane {i}: {r:?}");
            assert!(!close(c[i] as f64, r[i] as f64), "control: cos == sin at lane {i}");
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
            assert!(close(r[i] as f64, s::amd_sinf(xf[i]) as f64), "sinf lane {i}: {r:?}");
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
            assert!(close(r[i] as f64, s::amd_sinf(xf[i]) as f64), "sinf lane {i}: {r:?}");
        }
    }
}
