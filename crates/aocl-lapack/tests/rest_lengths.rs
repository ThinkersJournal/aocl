//! (23 of these 34 short cases were access violations before the checks.) Guard-page harness for the aocl-lapack functions the generated harness does not reach: the band solvers `gbsv` / `gbtrf` / `gbtrs` and the
//! direct trait `geev` (real f64 and complex c64), `gesvdx`, `orgbr` / `ungbr`. Hand-written. Each case passes ONE buffer ONE ELEMENT short
//! at the end of a committed page before a PAGE_NOACCESS page (a fault = HOLE); one process per test (tools/safe-api-harness.py); the control
//! (right sizes) must succeed. Windows only; Linux unmeasured.
#![cfg(windows)]
use aocl_lapack::*;
use core::ffi::c_void;

extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
    fn VirtualFree(addr: *mut c_void, size: usize, ty: u32) -> i32;
}
const PAGE: usize = 4096;
fn at_guard<T: Copy, R>(init: &[T], n: usize, f: impl FnOnce(&mut [T]) -> R) -> R {
    assert!(n <= init.len() && n * core::mem::size_of::<T>() <= PAGE);
    unsafe {
        let base = VirtualAlloc(core::ptr::null_mut(), 2 * PAGE, 0x3000, 0x04);
        assert!(!base.is_null());
        let mut old = 0u32;
        assert_ne!(
            VirtualProtect(
                (base as *mut u8).add(PAGE) as *mut c_void,
                PAGE,
                0x01,
                &mut old
            ),
            0
        );
        let start = (base as *mut u8).add(PAGE - n * core::mem::size_of::<T>()) as *mut T;
        for (i, v) in init.iter().take(n).enumerate() {
            start.add(i).write(*v);
        }
        let r = f(core::slice::from_raw_parts_mut(start, n));
        VirtualFree(base, 0, 0x8000);
        r
    }
}
macro_rules! case {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            let r = $body.map(|_| ());
            assert!(
                matches!(r, Err(aocl_error::Error::InvalidArgument(_))),
                "{}: {:?}",
                stringify!($name),
                r
            );
        }
    };
}
macro_rules! control {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            $body.unwrap();
        }
    };
}
const N: usize = 4;
const KL: usize = 1;
const KU: usize = 1;
const LDAB: usize = 2 * KL + KU + 1; // 4: row-major `ab` is N x LDAB
fn ab<T: From<f64> + Copy>() -> Vec<T> {
    (0..N * LDAB)
        .map(|i| T::from(if i % LDAB == KL + KU { 4.0 } else { 0.5 }))
        .collect()
}
fn one<T: From<f64> + Copy>(n: usize) -> Vec<T> {
    vec![T::from(1.0); n]
}
const IPIV: [Int; N] = [0; N];

control!(gbsv_f64_control, {
    let mut a = ab::<f64>();
    let mut ip = IPIV;
    let mut b = one::<f64>(N);
    <f64 as Scalar>::gbsv(
        Layout::RowMajor,
        N,
        KL,
        KU,
        1,
        &mut a,
        LDAB,
        &mut ip,
        &mut b,
        1,
    )
});
case!(gbsv_f64_short_ab, {
    let (mut ip, mut b) = (IPIV, one::<f64>(N));
    at_guard(&ab::<f64>(), N * LDAB - 1, |a| {
        <f64 as Scalar>::gbsv(Layout::RowMajor, N, KL, KU, 1, a, LDAB, &mut ip, &mut b, 1)
    })
});
case!(gbsv_f64_short_ipiv, {
    let (mut a, mut b) = (ab::<f64>(), one::<f64>(N));
    at_guard(&IPIV, N - 1, |ip| {
        <f64 as Scalar>::gbsv(Layout::RowMajor, N, KL, KU, 1, &mut a, LDAB, ip, &mut b, 1)
    })
});
case!(gbsv_f64_short_b, {
    let (mut a, mut ip) = (ab::<f64>(), IPIV);
    at_guard(&one::<f64>(N), N - 1, |b| {
        <f64 as Scalar>::gbsv(Layout::RowMajor, N, KL, KU, 1, &mut a, LDAB, &mut ip, b, 1)
    })
});
case!(gbsv_f64_small_ldab, {
    let (mut ip, mut b) = (IPIV, one::<f64>(N));
    at_guard(&ab::<f64>(), N * (LDAB - 1), |a| {
        <f64 as Scalar>::gbsv(
            Layout::RowMajor,
            N,
            KL,
            KU,
            1,
            a,
            LDAB - 1,
            &mut ip,
            &mut b,
            1,
        )
    })
});
control!(gbtrf_f64_control, {
    let mut a = ab::<f64>();
    let mut ip = IPIV;
    <f64 as Scalar>::gbtrf(Layout::RowMajor, N, N, KL, KU, &mut a, LDAB, &mut ip)
});
case!(gbtrf_f64_short_ab, {
    let mut ip = IPIV;
    at_guard(&ab::<f64>(), N * LDAB - 1, |a| {
        <f64 as Scalar>::gbtrf(Layout::RowMajor, N, N, KL, KU, a, LDAB, &mut ip)
    })
});
case!(gbtrf_f64_short_ipiv, {
    let mut a = ab::<f64>();
    at_guard(&IPIV, N - 1, |ip| {
        <f64 as Scalar>::gbtrf(Layout::RowMajor, N, N, KL, KU, &mut a, LDAB, ip)
    })
});
fn fact_f64() -> (Vec<f64>, [Int; N]) {
    let mut a = ab::<f64>();
    let mut ip = IPIV;
    <f64 as Scalar>::gbtrf(Layout::RowMajor, N, N, KL, KU, &mut a, LDAB, &mut ip).unwrap();
    (a, ip)
}
control!(gbtrs_f64_control, {
    let (a, ip) = fact_f64();
    let mut b = one::<f64>(N);
    <f64 as Scalar>::gbtrs(
        Layout::RowMajor,
        Trans::No,
        N,
        KL,
        KU,
        1,
        &a,
        LDAB,
        &ip,
        &mut b,
        1,
    )
});
case!(gbtrs_f64_short_ab, {
    let (a0, ip) = fact_f64();
    let mut b = one::<f64>(N);
    at_guard(&a0, N * LDAB - 1, |a| {
        <f64 as Scalar>::gbtrs(
            Layout::RowMajor,
            Trans::No,
            N,
            KL,
            KU,
            1,
            a,
            LDAB,
            &ip,
            &mut b,
            1,
        )
    })
});
case!(gbtrs_f64_short_ipiv, {
    let (a, ip0) = fact_f64();
    let mut b = one::<f64>(N);
    at_guard(&ip0, N - 1, |ip| {
        <f64 as Scalar>::gbtrs(
            Layout::RowMajor,
            Trans::No,
            N,
            KL,
            KU,
            1,
            &a,
            LDAB,
            ip,
            &mut b,
            1,
        )
    })
});
case!(gbtrs_f64_short_b, {
    let (a, ip) = fact_f64();
    at_guard(&one::<f64>(N), N - 1, |b| {
        <f64 as Scalar>::gbtrs(
            Layout::RowMajor,
            Trans::No,
            N,
            KL,
            KU,
            1,
            &a,
            LDAB,
            &ip,
            b,
            1,
        )
    })
});

control!(gbsv_c64_control, {
    let mut a = ab::<Complex64>();
    let mut ip = IPIV;
    let mut b = one::<Complex64>(N);
    <Complex64 as Scalar>::gbsv(
        Layout::RowMajor,
        N,
        KL,
        KU,
        1,
        &mut a,
        LDAB,
        &mut ip,
        &mut b,
        1,
    )
});
case!(gbsv_c64_short_ab, {
    let (mut ip, mut b) = (IPIV, one::<Complex64>(N));
    at_guard(&ab::<Complex64>(), N * LDAB - 1, |a| {
        <Complex64 as Scalar>::gbsv(Layout::RowMajor, N, KL, KU, 1, a, LDAB, &mut ip, &mut b, 1)
    })
});
case!(gbsv_c64_short_ipiv, {
    let (mut a, mut b) = (ab::<Complex64>(), one::<Complex64>(N));
    at_guard(&IPIV, N - 1, |ip| {
        <Complex64 as Scalar>::gbsv(Layout::RowMajor, N, KL, KU, 1, &mut a, LDAB, ip, &mut b, 1)
    })
});
case!(gbsv_c64_short_b, {
    let (mut a, mut ip) = (ab::<Complex64>(), IPIV);
    at_guard(&one::<Complex64>(N), N - 1, |b| {
        <Complex64 as Scalar>::gbsv(Layout::RowMajor, N, KL, KU, 1, &mut a, LDAB, &mut ip, b, 1)
    })
});
case!(gbsv_c64_small_ldab, {
    let (mut ip, mut b) = (IPIV, one::<Complex64>(N));
    at_guard(&ab::<Complex64>(), N * (LDAB - 1), |a| {
        <Complex64 as Scalar>::gbsv(
            Layout::RowMajor,
            N,
            KL,
            KU,
            1,
            a,
            LDAB - 1,
            &mut ip,
            &mut b,
            1,
        )
    })
});
control!(gbtrf_c64_control, {
    let mut a = ab::<Complex64>();
    let mut ip = IPIV;
    <Complex64 as Scalar>::gbtrf(Layout::RowMajor, N, N, KL, KU, &mut a, LDAB, &mut ip)
});
case!(gbtrf_c64_short_ab, {
    let mut ip = IPIV;
    at_guard(&ab::<Complex64>(), N * LDAB - 1, |a| {
        <Complex64 as Scalar>::gbtrf(Layout::RowMajor, N, N, KL, KU, a, LDAB, &mut ip)
    })
});
case!(gbtrf_c64_short_ipiv, {
    let mut a = ab::<Complex64>();
    at_guard(&IPIV, N - 1, |ip| {
        <Complex64 as Scalar>::gbtrf(Layout::RowMajor, N, N, KL, KU, &mut a, LDAB, ip)
    })
});
fn fact_c64() -> (Vec<Complex64>, [Int; N]) {
    let mut a = ab::<Complex64>();
    let mut ip = IPIV;
    <Complex64 as Scalar>::gbtrf(Layout::RowMajor, N, N, KL, KU, &mut a, LDAB, &mut ip).unwrap();
    (a, ip)
}
control!(gbtrs_c64_control, {
    let (a, ip) = fact_c64();
    let mut b = one::<Complex64>(N);
    <Complex64 as Scalar>::gbtrs(
        Layout::RowMajor,
        Trans::No,
        N,
        KL,
        KU,
        1,
        &a,
        LDAB,
        &ip,
        &mut b,
        1,
    )
});
case!(gbtrs_c64_short_ab, {
    let (a0, ip) = fact_c64();
    let mut b = one::<Complex64>(N);
    at_guard(&a0, N * LDAB - 1, |a| {
        <Complex64 as Scalar>::gbtrs(
            Layout::RowMajor,
            Trans::No,
            N,
            KL,
            KU,
            1,
            a,
            LDAB,
            &ip,
            &mut b,
            1,
        )
    })
});
case!(gbtrs_c64_short_ipiv, {
    let (a, ip0) = fact_c64();
    let mut b = one::<Complex64>(N);
    at_guard(&ip0, N - 1, |ip| {
        <Complex64 as Scalar>::gbtrs(
            Layout::RowMajor,
            Trans::No,
            N,
            KL,
            KU,
            1,
            &a,
            LDAB,
            ip,
            &mut b,
            1,
        )
    })
});
case!(gbtrs_c64_short_b, {
    let (a, ip) = fact_c64();
    at_guard(&one::<Complex64>(N), N - 1, |b| {
        <Complex64 as Scalar>::gbtrs(
            Layout::RowMajor,
            Trans::No,
            N,
            KL,
            KU,
            1,
            &a,
            LDAB,
            &ip,
            b,
            1,
        )
    })
});

fn mat<T: From<f64> + Copy>() -> Vec<T> {
    (0..N * N)
        .map(|k| {
            T::from(if k % N == k / N {
                3.0 + (k % N) as f64
            } else {
                0.25
            })
        })
        .collect()
}
control!(geev_f64_control, {
    let (mut a, mut wr, mut wi, mut vl, mut vr) = (
        mat::<f64>(),
        vec![0.0; N],
        vec![0.0; N],
        vec![0.0; N * N],
        vec![0.0; N * N],
    );
    <f64 as RealScalar>::geev(
        Layout::RowMajor,
        Compute::ValuesAndVectors,
        Compute::ValuesAndVectors,
        N,
        &mut a,
        N,
        &mut wr,
        &mut wi,
        &mut vl,
        N,
        &mut vr,
        N,
    )
});
case!(geev_f64_short_a, {
    let (mut wr, mut wi, mut vl, mut vr) = (
        vec![0.0; N],
        vec![0.0; N],
        vec![0.0; N * N],
        vec![0.0; N * N],
    );
    at_guard(&mat::<f64>(), N * N - 1, |a| {
        <f64 as RealScalar>::geev(
            Layout::RowMajor,
            Compute::ValuesAndVectors,
            Compute::ValuesAndVectors,
            N,
            a,
            N,
            &mut wr,
            &mut wi,
            &mut vl,
            N,
            &mut vr,
            N,
        )
    })
});
case!(geev_f64_short_wr, {
    let (mut a, mut wi, mut vl, mut vr) = (
        mat::<f64>(),
        vec![0.0; N],
        vec![0.0; N * N],
        vec![0.0; N * N],
    );
    at_guard(&[0.0f64; N], N - 1, |wr| {
        <f64 as RealScalar>::geev(
            Layout::RowMajor,
            Compute::ValuesAndVectors,
            Compute::ValuesAndVectors,
            N,
            &mut a,
            N,
            wr,
            &mut wi,
            &mut vl,
            N,
            &mut vr,
            N,
        )
    })
});
case!(geev_f64_short_wi, {
    let (mut a, mut wr, mut vl, mut vr) = (
        mat::<f64>(),
        vec![0.0; N],
        vec![0.0; N * N],
        vec![0.0; N * N],
    );
    at_guard(&[0.0f64; N], N - 1, |wi| {
        <f64 as RealScalar>::geev(
            Layout::RowMajor,
            Compute::ValuesAndVectors,
            Compute::ValuesAndVectors,
            N,
            &mut a,
            N,
            &mut wr,
            wi,
            &mut vl,
            N,
            &mut vr,
            N,
        )
    })
});
case!(geev_f64_short_vl, {
    let (mut a, mut wr, mut wi, mut vr) =
        (mat::<f64>(), vec![0.0; N], vec![0.0; N], vec![0.0; N * N]);
    at_guard(&[0.0f64; N * N], N * N - 1, |vl| {
        <f64 as RealScalar>::geev(
            Layout::RowMajor,
            Compute::ValuesAndVectors,
            Compute::ValuesAndVectors,
            N,
            &mut a,
            N,
            &mut wr,
            &mut wi,
            vl,
            N,
            &mut vr,
            N,
        )
    })
});
case!(geev_f64_short_vr, {
    let (mut a, mut wr, mut wi, mut vl) =
        (mat::<f64>(), vec![0.0; N], vec![0.0; N], vec![0.0; N * N]);
    at_guard(&[0.0f64; N * N], N * N - 1, |vr| {
        <f64 as RealScalar>::geev(
            Layout::RowMajor,
            Compute::ValuesAndVectors,
            Compute::ValuesAndVectors,
            N,
            &mut a,
            N,
            &mut wr,
            &mut wi,
            &mut vl,
            N,
            vr,
            N,
        )
    })
});
control!(geev_c64_control, {
    let (mut a, mut w, mut vl, mut vr) = (
        mat::<Complex64>(),
        vec![Complex64::from(0.0); N],
        vec![Complex64::from(0.0); N * N],
        vec![Complex64::from(0.0); N * N],
    );
    <Complex64 as ComplexScalar>::geev(
        Layout::RowMajor,
        Compute::ValuesAndVectors,
        Compute::ValuesAndVectors,
        N,
        &mut a,
        N,
        &mut w,
        &mut vl,
        N,
        &mut vr,
        N,
    )
});
case!(geev_c64_short_a, {
    let (mut w, mut vl, mut vr) = (
        vec![Complex64::from(0.0); N],
        vec![Complex64::from(0.0); N * N],
        vec![Complex64::from(0.0); N * N],
    );
    at_guard(&mat::<Complex64>(), N * N - 1, |a| {
        <Complex64 as ComplexScalar>::geev(
            Layout::RowMajor,
            Compute::ValuesAndVectors,
            Compute::ValuesAndVectors,
            N,
            a,
            N,
            &mut w,
            &mut vl,
            N,
            &mut vr,
            N,
        )
    })
});
case!(geev_c64_short_w, {
    let (mut a, mut vl, mut vr) = (
        mat::<Complex64>(),
        vec![Complex64::from(0.0); N * N],
        vec![Complex64::from(0.0); N * N],
    );
    at_guard(&[Complex64::from(0.0); N], N - 1, |w| {
        <Complex64 as ComplexScalar>::geev(
            Layout::RowMajor,
            Compute::ValuesAndVectors,
            Compute::ValuesAndVectors,
            N,
            &mut a,
            N,
            w,
            &mut vl,
            N,
            &mut vr,
            N,
        )
    })
});
case!(geev_c64_short_vl, {
    let (mut a, mut w, mut vr) = (
        mat::<Complex64>(),
        vec![Complex64::from(0.0); N],
        vec![Complex64::from(0.0); N * N],
    );
    at_guard(&[Complex64::from(0.0); N * N], N * N - 1, |vl| {
        <Complex64 as ComplexScalar>::geev(
            Layout::RowMajor,
            Compute::ValuesAndVectors,
            Compute::ValuesAndVectors,
            N,
            &mut a,
            N,
            &mut w,
            vl,
            N,
            &mut vr,
            N,
        )
    })
});
case!(geev_c64_short_vr, {
    let (mut a, mut w, mut vl) = (
        mat::<Complex64>(),
        vec![Complex64::from(0.0); N],
        vec![Complex64::from(0.0); N * N],
    );
    at_guard(&[Complex64::from(0.0); N * N], N * N - 1, |vr| {
        <Complex64 as ComplexScalar>::geev(
            Layout::RowMajor,
            Compute::ValuesAndVectors,
            Compute::ValuesAndVectors,
            N,
            &mut a,
            N,
            &mut w,
            &mut vl,
            N,
            vr,
            N,
        )
    })
});
// gesvdx f64: 4 x 3, ns <= 3; u is 4 x 3 (ldu 3), vt is 3 x 3 (ldvt 3), superb 12 * min(m, n) = 36
fn m43() -> Vec<f64> {
    (0..12)
        .map(|k| 1.0 + (k % 5) as f64 + (k / 3) as f64 * 0.5)
        .collect()
}
control!(gesvdx_control, {
    let (mut a, mut s, mut u, mut vt, mut sb) = (
        m43(),
        vec![0.0; 3],
        vec![0.0; 12],
        vec![0.0; 9],
        vec![0 as Int; 36],
    );
    dgesvdx(
        true,
        true,
        Range::All,
        4,
        3,
        &mut a,
        3,
        0.0,
        0.0,
        0,
        0,
        &mut s,
        &mut u,
        3,
        &mut vt,
        3,
        &mut sb,
    )
});
case!(gesvdx_short_a, {
    let (mut s, mut u, mut vt, mut sb) = (
        vec![0.0; 3],
        vec![0.0; 12],
        vec![0.0; 9],
        vec![0 as Int; 36],
    );
    at_guard(&m43(), 11, |a| {
        dgesvdx(
            true,
            true,
            Range::All,
            4,
            3,
            a,
            3,
            0.0,
            0.0,
            0,
            0,
            &mut s,
            &mut u,
            3,
            &mut vt,
            3,
            &mut sb,
        )
    })
});
case!(gesvdx_short_s, {
    let (mut a, mut u, mut vt, mut sb) = (m43(), vec![0.0; 12], vec![0.0; 9], vec![0 as Int; 36]);
    at_guard(&[0.0f64; 3], 2, |s| {
        dgesvdx(
            true,
            true,
            Range::All,
            4,
            3,
            &mut a,
            3,
            0.0,
            0.0,
            0,
            0,
            s,
            &mut u,
            3,
            &mut vt,
            3,
            &mut sb,
        )
    })
});
case!(gesvdx_short_u, {
    let (mut a, mut s, mut vt, mut sb) = (m43(), vec![0.0; 3], vec![0.0; 9], vec![0 as Int; 36]);
    at_guard(&[0.0f64; 12], 11, |u| {
        dgesvdx(
            true,
            true,
            Range::All,
            4,
            3,
            &mut a,
            3,
            0.0,
            0.0,
            0,
            0,
            &mut s,
            u,
            3,
            &mut vt,
            3,
            &mut sb,
        )
    })
});
case!(gesvdx_short_vt, {
    let (mut a, mut s, mut u, mut sb) = (m43(), vec![0.0; 3], vec![0.0; 12], vec![0 as Int; 36]);
    at_guard(&[0.0f64; 9], 8, |vt| {
        dgesvdx(
            true,
            true,
            Range::All,
            4,
            3,
            &mut a,
            3,
            0.0,
            0.0,
            0,
            0,
            &mut s,
            &mut u,
            3,
            vt,
            3,
            &mut sb,
        )
    })
});
case!(gesvdx_short_superb, {
    let (mut a, mut s, mut u, mut vt) = (m43(), vec![0.0; 3], vec![0.0; 12], vec![0.0; 9]);
    at_guard(&[0 as Int; 36], 35, |sb| {
        dgesvdx(
            true,
            true,
            Range::All,
            4,
            3,
            &mut a,
            3,
            0.0,
            0.0,
            0,
            0,
            &mut s,
            &mut u,
            3,
            &mut vt,
            3,
            sb,
        )
    })
});
// orgbr 'Q' on the reflectors of a 4 x 3 gebrd: a is 4 x 3 (lda 3), tau = tauq (3)
fn bidiag() -> (Vec<f64>, Vec<f64>) {
    let (mut a, mut d, mut e, mut tq, mut tp) = (
        m43(),
        vec![0.0; 3],
        vec![0.0; 2],
        vec![0.0; 3],
        vec![0.0; 3],
    );
    dgebrd(4, 3, &mut a, 3, &mut d, &mut e, &mut tq, &mut tp).unwrap();
    (a, tq)
}
control!(orgbr_control, {
    let (mut a, tq) = bidiag();
    dorgbr(BidiagVect::Q, 4, 3, 3, &mut a, 3, &tq)
});
case!(orgbr_short_a, {
    let (a0, tq) = bidiag();
    at_guard(&a0, 11, |a| dorgbr(BidiagVect::Q, 4, 3, 3, a, 3, &tq))
});
case!(orgbr_short_tau, {
    let (mut a, tq) = bidiag();
    at_guard(&tq, 2, |t| dorgbr(BidiagVect::Q, 4, 3, 3, &mut a, 3, t))
});
