//! Result-checking tests for the safe wrappers that the in-crate tests do not touch (run on the real AOCL-LAPACK).
//!
//! Expected values never come from the library: they are fixed by construction (pick `x`, form `b = A x` with the serial
//! reference product below, solve, compare `x`), by closed form (a matrix `H D H` with `H` a Householder reflector has the
//! eigenvalues `D` exactly; `A = U S V^H` has the singular values `S`), or by invariants (`Q^H Q = I`, `A = Q R`,
//! `||A v - w v||`). All matrices are small, dense, row-major, and checked against a stated tolerance (`Num::TOL`).

use aocl_lapack::*;
use aocl_types::Diag;
use std::ops::{Add, Mul, Sub};

// ---------------------------------------------------------------------------------------------------------------------
// Reference arithmetic (complex f64), independent of the library
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
struct C(f64, f64);
impl Add for C {
    type Output = C;
    fn add(self, o: C) -> C {
        C(self.0 + o.0, self.1 + o.1)
    }
}
impl Sub for C {
    type Output = C;
    fn sub(self, o: C) -> C {
        C(self.0 - o.0, self.1 - o.1)
    }
}
impl Mul for C {
    type Output = C;
    fn mul(self, o: C) -> C {
        C(self.0 * o.0 - self.1 * o.1, self.0 * o.1 + self.1 * o.0)
    }
}
impl C {
    fn conj(self) -> C {
        C(self.0, -self.1)
    }
    fn abs(self) -> f64 {
        self.0.hypot(self.1)
    }
    fn sc(self, k: f64) -> C {
        C(self.0 * k, self.1 * k)
    }
    /// `self / o`
    fn div(self, o: C) -> C {
        let d = o.0 * o.0 + o.1 * o.1;
        let n = self * o.conj();
        C(n.0 / d, n.1 / d)
    }
}
const ZERO: C = C(0.0, 0.0);
const ONE: C = C(1.0, 0.0);

/// The four element types: conversion to/from the reference type plus the tolerance for that precision.
trait Num: Copy + std::fmt::Debug + 'static {
    type R: Num;
    const CPLX: bool;
    const TOL: f64;
    fn mk(re: f64, im: f64) -> Self;
    fn parts(self) -> (f64, f64);
}
impl Num for f64 {
    type R = f64;
    const CPLX: bool = false;
    const TOL: f64 = 1e-10;
    fn mk(re: f64, _: f64) -> f64 {
        re
    }
    fn parts(self) -> (f64, f64) {
        (self, 0.0)
    }
}
impl Num for f32 {
    type R = f32;
    const CPLX: bool = false;
    const TOL: f64 = 3e-4;
    fn mk(re: f64, _: f64) -> f32 {
        re as f32
    }
    fn parts(self) -> (f64, f64) {
        (self as f64, 0.0)
    }
}
impl Num for Complex64 {
    type R = f64;
    const CPLX: bool = true;
    const TOL: f64 = 1e-10;
    fn mk(re: f64, im: f64) -> Complex64 {
        Complex64::new(re, im)
    }
    fn parts(self) -> (f64, f64) {
        (self.re, self.im)
    }
}
impl Num for Complex32 {
    type R = f32;
    const CPLX: bool = true;
    const TOL: f64 = 3e-4;
    fn mk(re: f64, im: f64) -> Complex32 {
        Complex32::new(re as f32, im as f32)
    }
    fn parts(self) -> (f64, f64) {
        (self.re as f64, self.im as f64)
    }
}

fn cv<T: Num>(v: &[T]) -> Vec<C> {
    v.iter()
        .map(|x| {
            let (a, b) = x.parts();
            C(a, b)
        })
        .collect()
}
fn tv<T: Num>(v: &[C]) -> Vec<T> {
    v.iter().map(|c| T::mk(c.0, c.1)).collect()
}
fn zeros<T: Num>(n: usize) -> Vec<T> {
    vec![T::mk(0.0, 0.0); n]
}
fn rr<T: Num>(x: f64) -> T::R {
    T::R::mk(x, 0.0)
}
fn re<R: Num>(x: R) -> f64 {
    x.parts().0
}
fn th<T: Num>() -> Trans {
    if T::CPLX {
        Trans::C
    } else {
        Trans::T
    }
}

fn mm(a: &[C], m: usize, k: usize, b: &[C], n: usize) -> Vec<C> {
    assert_eq!((a.len(), b.len()), (m * k, k * n));
    let mut c = vec![ZERO; m * n];
    for i in 0..m {
        for j in 0..n {
            c[i * n + j] = (0..k).fold(ZERO, |s, p| s + a[i * k + p] * b[p * n + j]);
        }
    }
    c
}
/// Conjugate transpose of an `m x n` matrix (result `n x m`).
fn adj(a: &[C], m: usize, n: usize) -> Vec<C> {
    let mut r = vec![ZERO; m * n];
    for i in 0..m {
        for j in 0..n {
            r[j * m + i] = a[i * n + j].conj();
        }
    }
    r
}
fn eye(n: usize) -> Vec<C> {
    let mut e = vec![ZERO; n * n];
    for i in 0..n {
        e[i * n + i] = ONE;
    }
    e
}
fn maxdiff(a: &[C], b: &[C]) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b)
        .map(|(x, y)| (*x - *y).abs())
        .fold(0.0, f64::max)
}
fn close(what: &str, got: &[C], want: &[C], tol: f64) {
    let d = maxdiff(got, want);
    assert!(d <= tol, "{what}: max|got - want| = {d} > {tol}");
}
fn near(what: &str, got: f64, want: f64, tol: f64) {
    assert!(
        (got - want).abs() <= tol,
        "{what}: |{got} - {want}| > {tol}"
    );
}
/// `mm` scaled: `sc * a` entrywise.
fn scale(a: &[C], k: C) -> Vec<C> {
    a.iter().map(|x| *x * k).collect()
}
fn sub_m(a: &[C], b: &[C]) -> Vec<C> {
    a.iter().zip(b).map(|(x, y)| *x - *y).collect()
}
/// Column `j` of an `m x n` matrix.
fn col(a: &[C], m: usize, n: usize, j: usize) -> Vec<C> {
    (0..m).map(|i| a[i * n + j]).collect()
}
/// Gauss-Jordan inverse with partial pivoting.
fn inv(a: &[C], n: usize) -> Vec<C> {
    let mut m = a.to_vec();
    let mut r = eye(n);
    for c in 0..n {
        let p = (c..n)
            .max_by(|&x, &y| m[x * n + c].abs().total_cmp(&m[y * n + c].abs()))
            .unwrap();
        for j in 0..n {
            m.swap(c * n + j, p * n + j);
            r.swap(c * n + j, p * n + j);
        }
        let d = m[c * n + c];
        for j in 0..n {
            m[c * n + j] = m[c * n + j].div(d);
            r[c * n + j] = r[c * n + j].div(d);
        }
        for i in 0..n {
            if i != c {
                let f = m[i * n + c];
                for j in 0..n {
                    m[i * n + j] = m[i * n + j] - f * m[c * n + j];
                    r[i * n + j] = r[i * n + j] - f * r[c * n + j];
                }
            }
        }
    }
    r
}
/// Entrywise-abs matrix norms (`1`, `inf`, `max`, `fro`) of an `m x n` matrix.
fn ref_norm(a: &[C], m: usize, n: usize, which: Norm) -> f64 {
    let ab = |i: usize, j: usize| a[i * n + j].abs();
    match which {
        Norm::Max => (0..m)
            .flat_map(|i| (0..n).map(move |j| (i, j)))
            .map(|(i, j)| ab(i, j))
            .fold(0.0, f64::max),
        Norm::One => (0..n)
            .map(|j| (0..m).map(|i| ab(i, j)).sum::<f64>())
            .fold(0.0, f64::max),
        Norm::Inf => (0..m)
            .map(|i| (0..n).map(|j| ab(i, j)).sum::<f64>())
            .fold(0.0, f64::max),
        Norm::Frobenius => a.iter().map(|x| x.abs() * x.abs()).sum::<f64>().sqrt(),
    }
}

/// Deterministic, well-conditioned test matrix (diagonal boost), imaginary parts only when `cplx`.
fn gen(m: usize, n: usize, seed: usize, cplx: bool) -> Vec<C> {
    let mut a = Vec::with_capacity(m * n);
    for i in 0..m {
        for j in 0..n {
            let t = ((i * 5 + j * 3 + seed * 7) % 11) as f64;
            let reall = t / 5.0 - 1.0 + if i == j { 3.0 } else { 0.0 };
            let im = if cplx {
                (((i * 3 + j * 7 + seed) % 9) as f64) / 4.0 - 1.0
            } else {
                0.0
            };
            a.push(C(reall, im));
        }
    }
    a
}
/// Householder reflector `I - 2 u u^H / (u^H u)`: unitary and Hermitian.
fn hh(n: usize, seed: usize, cplx: bool) -> Vec<C> {
    let u = gen(n, 1, seed, cplx);
    let uu = mm(&u, n, 1, &adj(&u, n, 1), n);
    let nrm: f64 = u.iter().map(|x| x.abs() * x.abs()).sum();
    let mut h = eye(n);
    for k in 0..n * n {
        h[k] = h[k] - uu[k].sc(2.0 / nrm);
    }
    h
}
fn diag_m(d: &[C]) -> Vec<C> {
    let n = d.len();
    let mut m = vec![ZERO; n * n];
    for i in 0..n {
        m[i * n + i] = d[i];
    }
    m
}
/// Hermitian (symmetric if real) matrix with exactly the eigenvalues `d`: `H D H`.
fn spec_herm(d: &[f64], seed: usize, cplx: bool) -> Vec<C> {
    let n = d.len();
    let h = hh(n, seed, cplx);
    mm(
        &mm(
            &h,
            n,
            n,
            &diag_m(&d.iter().map(|x| C(*x, 0.0)).collect::<Vec<_>>()),
            n,
        ),
        n,
        n,
        &h,
        n,
    )
}
/// Overwrite everything the routine must not read (the strict opposite triangle) with garbage.
fn poison(a: &mut [C], n: usize, uplo: Uplo) {
    for i in 0..n {
        for j in 0..n {
            let skip = match uplo {
                Uplo::Upper => j < i,
                Uplo::Lower => j > i,
            };
            if skip {
                a[i * n + j] = C(1e3, 1e3);
            }
        }
    }
}
/// Dense triangular matrix from the stored triangle (unit diagonal when `Diag::Unit`).
fn tri_dense(a: &[C], n: usize, uplo: Uplo, diag: Diag) -> Vec<C> {
    let mut t = vec![ZERO; n * n];
    for i in 0..n {
        for j in 0..n {
            let inside = match uplo {
                Uplo::Upper => j >= i,
                Uplo::Lower => j <= i,
            };
            if inside {
                t[i * n + j] = if i == j && matches!(diag, Diag::Unit) {
                    ONE
                } else {
                    a[i * n + j]
                };
            }
        }
    }
    t
}
/// Full matrix from one stored triangle; `herm` mirrors with conjugation, otherwise symmetric mirror.
fn mirror(a: &[C], n: usize, uplo: Uplo, herm: bool) -> Vec<C> {
    let mut f = a.to_vec();
    for i in 0..n {
        for j in 0..n {
            let stored = match uplo {
                Uplo::Upper => j >= i,
                Uplo::Lower => j <= i,
            };
            if !stored {
                let s = a[j * n + i];
                f[i * n + j] = if herm { s.conj() } else { s };
            }
        }
    }
    if herm {
        for i in 0..n {
            f[i * n + i].1 = 0.0;
        }
    }
    f
}
/// `m x n` matrix copied into storage with row stride `ld`, padding filled with a sentinel.
fn pad(a: &[C], m: usize, n: usize, ld: usize, fill: C) -> Vec<C> {
    let mut p = vec![fill; m * ld];
    for i in 0..m {
        for j in 0..n {
            p[i * ld + j] = a[i * n + j];
        }
    }
    p
}
/// Every wanted value is matched by a distinct got value (eigenvalue sets, order-free).
fn same_set(what: &str, got: &[C], want: &[C], tol: f64) {
    assert_eq!(got.len(), want.len(), "{what}: length");
    let mut used = vec![false; got.len()];
    for w in want {
        let k = (0..got.len()).find(|&k| !used[k] && (got[k] - *w).abs() <= tol);
        match k {
            Some(k) => used[k] = true,
            None => panic!("{what}: wanted {w:?} not in {got:?}"),
        }
    }
}
fn is_ident(a: &[C], n: usize) -> f64 {
    maxdiff(a, &eye(n))
}
/// Hessenberg part / generic helpers
fn upper_part(a: &[C], m: usize, n: usize) -> Vec<C> {
    let mut r = a.to_vec();
    for i in 0..m {
        for j in 0..n.min(i) {
            r[i * n + j] = ZERO;
        }
    }
    r
}

// ---------------------------------------------------------------------------------------------------------------------
// Triangular solve / inverse, Cholesky inverse (real precisions only)
// ---------------------------------------------------------------------------------------------------------------------

fn check_trtrs<T: Num>(
    f: impl Fn(Uplo, Trans, Diag, usize, usize, &[T], usize, &mut [T], usize) -> Result<()>,
) {
    let (n, nrhs) = (3, 2);
    let x = gen(n, nrhs, 2, false);
    for (uplo, trans, diag) in [
        (Uplo::Upper, Trans::No, Diag::NonUnit),
        (Uplo::Lower, Trans::T, Diag::Unit),
        (Uplo::Upper, Trans::T, Diag::NonUnit),
    ] {
        // `stored` has live values in the opposite triangle and (for Unit) on the diagonal: they must be ignored.
        let stored = gen(n, n, 1, false);
        let t = tri_dense(&stored, n, uplo, diag);
        let op = if trans == Trans::No { t } else { adj(&t, n, n) };
        let b = mm(&op, n, n, &x, nrhs);
        let (a_t, mut b_t) = (tv::<T>(&stored), tv::<T>(&b));
        f(uplo, trans, diag, n, nrhs, &a_t, n, &mut b_t, nrhs).unwrap();
        close(
            &format!("trtrs {uplo:?} {trans:?} {diag:?}"),
            &cv(&b_t),
            &x,
            T::TOL,
        );
    }
    // lda smaller than n is rejected (illegal argument), not read past.
    let a = tv::<T>(&gen(n, n, 1, false));
    let mut b = tv::<T>(&gen(n, 1, 2, false));
    assert!(f(
        Uplo::Upper,
        Trans::No,
        Diag::NonUnit,
        n,
        1,
        &a,
        1,
        &mut b,
        1
    )
    .is_err());
}
#[test]
fn t_trtrs_real() {
    check_trtrs::<f64>(dtrtrs);
    check_trtrs::<f32>(strtrs);
}

fn check_trtri<T: Num>(f: impl Fn(Uplo, Diag, usize, &mut [T], usize) -> Result<()>) {
    let n = 3;
    for (uplo, diag) in [
        (Uplo::Upper, Diag::NonUnit),
        (Uplo::Lower, Diag::NonUnit),
        (Uplo::Upper, Diag::Unit),
    ] {
        let stored = gen(n, n, 4, false);
        let t = tri_dense(&stored, n, uplo, diag);
        let mut a = tv::<T>(&stored);
        f(uplo, diag, n, &mut a, n).unwrap();
        let ti = tri_dense(&cv(&a), n, uplo, diag);
        close(
            &format!("trtri {uplo:?} {diag:?}: T * inv(T) = I"),
            &mm(&t, n, n, &ti, n),
            &eye(n),
            T::TOL,
        );
    }
    // a singular triangle (zero on the diagonal) reports the failing index
    let mut sing = tv::<T>(&[ONE, ONE, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ONE]);
    assert!(f(Uplo::Upper, Diag::NonUnit, 3, &mut sing, 3).is_err());
}
#[test]
fn t_trtri_real() {
    check_trtri::<f64>(dtrtri);
    check_trtri::<f32>(strtri);
}

fn spd(n: usize, seed: usize, cplx: bool) -> Vec<C> {
    let g = gen(n, n, seed, cplx);
    let mut a = mm(&adj(&g, n, n), n, n, &g, n);
    for i in 0..n {
        a[i * n + i] = a[i * n + i] + C(n as f64, 0.0);
    }
    a
}

fn check_potri<T: Num + Scalar>(f: impl Fn(Uplo, usize, &mut [T], usize) -> Result<()>) {
    let n = 3;
    for uplo in [Uplo::Upper, Uplo::Lower] {
        let a0 = spd(n, 1, false);
        let mut a = tv::<T>(&a0);
        potrf::<T>(uplo, n, &mut a).unwrap();
        f(uplo, n, &mut a, n).unwrap();
        let invf = mirror(&cv(&a), n, uplo, true);
        close(
            &format!("potri {uplo:?}: A * inv(A) = I"),
            &mm(&a0, n, n, &invf, n),
            &eye(n),
            T::TOL * 10.0,
        );
    }
}
#[test]
fn t_potri_real() {
    check_potri::<f64>(dpotri);
    check_potri::<f32>(spotri);
}

// ---------------------------------------------------------------------------------------------------------------------
// QR and LQ: form Q (orgqr/ungqr, orglq/unglq) and apply Q (ormqr/unmqr, ormlq/unmlq); gelqf
// ---------------------------------------------------------------------------------------------------------------------

fn check_qr<T: Num + Scalar>(
    orgqr: impl Fn(usize, usize, usize, &mut [T], usize, &[T]) -> Result<()>,
    ormqr: impl Fn(bool, Trans, usize, usize, usize, &[T], usize, &[T], &mut [T], usize) -> Result<()>,
) {
    let (m, n) = (4, 3);
    let a0 = gen(m, n, 3, T::CPLX);
    let mut a = tv::<T>(&a0);
    let mut tau = zeros::<T>(n);
    geqrf::<T>(m, n, &mut a, &mut tau).unwrap();
    let refl = a.clone();
    let r = upper_part(&cv(&a)[..n * n], n, n);
    // thin Q (m x n)
    let mut q = refl.clone();
    orgqr(m, n, n, &mut q, n, &tau).unwrap();
    let qc = cv(&q);
    close(
        "Q^H Q = I",
        &mm(&adj(&qc, m, n), n, m, &qc, n),
        &eye(n),
        T::TOL,
    );
    close("Q R = A", &mm(&qc, m, n, &r, n), &a0, T::TOL);
    // full Q (m x m) from the same reflectors
    let mut qf = zeros::<T>(m * m);
    for i in 0..m {
        for j in 0..n {
            qf[i * m + j] = refl[i * n + j];
        }
    }
    orgqr(m, m, n, &mut qf, m, &tau).unwrap();
    let qf = cv(&qf);
    close(
        "Qfull^H Qfull = I",
        &mm(&adj(&qf, m, m), m, m, &qf, m),
        &eye(m),
        T::TOL,
    );
    let qh = adj(&qf, m, m);
    let c0 = gen(m, 2, 5, T::CPLX);
    for (trans, op) in [(Trans::No, &qf), (th::<T>(), &qh)] {
        let mut c = tv::<T>(&c0);
        ormqr(true, trans, m, 2, n, &refl, n, &tau, &mut c, 2).unwrap();
        close(
            &format!("ormqr left {trans:?}"),
            &cv(&c),
            &mm(op, m, m, &c0, 2),
            T::TOL,
        );
    }
    let cr0 = gen(2, m, 6, T::CPLX);
    for (trans, op) in [(Trans::No, &qf), (th::<T>(), &qh)] {
        let mut c = tv::<T>(&cr0);
        ormqr(false, trans, 2, m, n, &refl, n, &tau, &mut c, m).unwrap();
        close(
            &format!("ormqr right {trans:?}"),
            &cv(&c),
            &mm(&cr0, 2, m, op, m),
            T::TOL,
        );
    }
}
#[test]
fn t_qr_family() {
    check_qr::<f64>(dorgqr, dormqr);
    check_qr::<f32>(sorgqr, sormqr);
    check_qr::<Complex64>(zungqr, zunmqr);
    check_qr::<Complex32>(cungqr, cunmqr);
}

fn check_lq<T: Num + Scalar>(
    orglq: impl Fn(usize, usize, usize, &mut [T], usize, &[T]) -> Result<()>,
    ormlq: impl Fn(bool, Trans, usize, usize, usize, &[T], usize, &[T], &mut [T], usize) -> Result<()>,
) {
    let (m, n) = (3, 4);
    let a0 = gen(m, n, 2, T::CPLX);
    let mut a = tv::<T>(&a0);
    let mut tau = zeros::<T>(m);
    gelqf::<T>(m, n, &mut a, &mut tau).unwrap();
    let refl = a.clone();
    // L = lower triangle of the leading m x m block
    let ac = cv(&a);
    let mut l = vec![ZERO; m * m];
    for i in 0..m {
        for j in 0..=i {
            l[i * m + j] = ac[i * n + j];
        }
    }
    let mut q = refl.clone();
    orglq(m, n, m, &mut q, n, &tau).unwrap();
    let qc = cv(&q);
    close(
        "Q Q^H = I",
        &mm(&qc, m, n, &adj(&qc, m, n), m),
        &eye(m),
        T::TOL,
    );
    close("L Q = A", &mm(&l, m, m, &qc, n), &a0, T::TOL);
    // full Q (n x n)
    let mut qf = zeros::<T>(n * n);
    qf[..m * n].copy_from_slice(&refl);
    orglq(n, n, m, &mut qf, n, &tau).unwrap();
    let qf = cv(&qf);
    close(
        "Qfull Qfull^H = I",
        &mm(&qf, n, n, &adj(&qf, n, n), n),
        &eye(n),
        T::TOL,
    );
    let qh = adj(&qf, n, n);
    let c0 = gen(n, 2, 5, T::CPLX);
    for (trans, op) in [(Trans::No, &qf), (th::<T>(), &qh)] {
        let mut c = tv::<T>(&c0);
        ormlq(true, trans, n, 2, m, &refl, n, &tau, &mut c, 2).unwrap();
        close(
            &format!("ormlq left {trans:?}"),
            &cv(&c),
            &mm(op, n, n, &c0, 2),
            T::TOL,
        );
    }
    let cr0 = gen(2, n, 6, T::CPLX);
    for (trans, op) in [(Trans::No, &qf), (th::<T>(), &qh)] {
        let mut c = tv::<T>(&cr0);
        ormlq(false, trans, 2, n, m, &refl, n, &tau, &mut c, n).unwrap();
        close(
            &format!("ormlq right {trans:?}"),
            &cv(&c),
            &mm(&cr0, 2, n, op, n),
            T::TOL,
        );
    }
}
#[test]
fn t_lq_family() {
    check_lq::<f64>(dorglq, dormlq);
    check_lq::<f32>(sorglq, sormlq);
    check_lq::<Complex64>(zunglq, zunmlq);
    check_lq::<Complex32>(cunglq, cunmlq);
}

#[test]
fn t_gelqf_dimension_mismatch() {
    // A slice shorter than m*n, and a tau shorter than min(m,n), are rejected before reaching LAPACK.
    let mut a = [1.0_f64; 5];
    let mut tau = [0.0_f64; 2];
    assert!(matches!(
        gelqf::<f64>(2, 3, &mut a, &mut tau),
        Err(Error::InvalidArgument(_))
    ));
    let mut a = [1.0_f32; 6];
    let mut tau = [0.0_f32; 1];
    assert!(matches!(
        gelqf::<f32>(2, 3, &mut a, &mut tau),
        Err(Error::InvalidArgument(_))
    ));
}

// ---------------------------------------------------------------------------------------------------------------------
// Non-symmetric eigenproblems: geev, gees, ggev
// ---------------------------------------------------------------------------------------------------------------------

/// `H T H` with `T` upper block-triangular: the spectrum is that of `T` exactly (`H` unitary Hermitian).
fn similar_to(t: &[C], n: usize, seed: usize, cplx: bool) -> Vec<C> {
    let h = hh(n, seed, cplx);
    mm(&mm(&h, n, n, t, n), n, n, &h, n)
}
fn real_block_t() -> Vec<C> {
    // eigenvalues: +i, -i (2x2 rotation block), 3
    [0.0, -1.0, 1.0, 1.0, 0.0, 2.0, 0.0, 0.0, 3.0]
        .iter()
        .map(|&x| C(x, 0.0))
        .collect()
}
fn real_block_eigs() -> Vec<C> {
    vec![C(0.0, 1.0), C(0.0, -1.0), C(3.0, 0.0)]
}
/// Real-geev output (`wr`, `wi`, packed real vector matrix) as complex eigenvectors (columns).
fn unpack_real_vecs(wi: &[C], v: &[C], n: usize) -> Vec<Vec<C>> {
    let mut out = Vec::new();
    for j in 0..n {
        let (cur, prev) = (
            col(v, n, n, j),
            if j > 0 { col(v, n, n, j - 1) } else { vec![] },
        );
        let next = if j + 1 < n {
            col(v, n, n, j + 1)
        } else {
            vec![]
        };
        out.push(if wi[j].0 == 0.0 {
            cur
        } else if wi[j].0 > 0.0 {
            cur.iter()
                .zip(&next)
                .map(|(r, i)| *r + C(0.0, 1.0) * *i)
                .collect()
        } else {
            prev.iter()
                .zip(&cur)
                .map(|(r, i)| *r - C(0.0, 1.0) * *i)
                .collect()
        });
    }
    out
}
fn vec_norm(v: &[C]) -> f64 {
    v.iter().map(|x| x.abs() * x.abs()).sum::<f64>().sqrt()
}
/// `A v = lam v` and `||v|| = 1` for every column; with `left`, `v^H A = lam v^H`.
fn check_eigpairs(a: &[C], n: usize, lams: &[C], vecs: &[Vec<C>], left: bool, tol: f64) {
    for (lam, v) in lams.iter().zip(vecs) {
        let r = if left {
            // v^H A = lam v^H  <=>  A^H v = conj(lam) v
            let w = mm(&adj(a, n, n), n, n, v, 1);
            w.iter()
                .zip(v)
                .map(|(x, y)| *x - lam.conj() * *y)
                .collect::<Vec<_>>()
        } else {
            let w = mm(a, n, n, v, 1);
            w.iter()
                .zip(v)
                .map(|(x, y)| *x - *lam * *y)
                .collect::<Vec<_>>()
        };
        assert!(
            vec_norm(&r) <= tol,
            "eigen residual {} for lambda {lam:?} (left={left})",
            vec_norm(&r)
        );
        near("eigenvector norm", vec_norm(v), 1.0, tol);
    }
}

fn check_geev_real<T: Num>(
    f: impl Fn(
        bool,
        bool,
        usize,
        &mut [T],
        usize,
        &mut [T],
        &mut [T],
        &mut [T],
        usize,
        &mut [T],
        usize,
    ) -> Result<()>,
) {
    let n = 3;
    let a0 = similar_to(&real_block_t(), n, 3, false);
    let mut a = tv::<T>(&a0);
    let (mut wr, mut wi) = (zeros::<T>(n), zeros::<T>(n));
    let (mut vl, mut vr) = (zeros::<T>(n * n), zeros::<T>(n * n));
    f(
        true, true, n, &mut a, n, &mut wr, &mut wi, &mut vl, n, &mut vr, n,
    )
    .unwrap();
    let (wrc, wic) = (cv(&wr), cv(&wi));
    let lams: Vec<C> = (0..n).map(|j| C(wrc[j].0, wic[j].0)).collect();
    same_set("geev eigenvalues", &lams, &real_block_eigs(), T::TOL);
    check_eigpairs(
        &a0,
        n,
        &lams,
        &unpack_real_vecs(&wic, &cv(&vr), n),
        false,
        T::TOL * 5.0,
    );
    check_eigpairs(
        &a0,
        n,
        &lams,
        &unpack_real_vecs(&wic, &cv(&vl), n),
        true,
        T::TOL * 5.0,
    );
    // values-only call gives the same spectrum
    let mut a = tv::<T>(&a0);
    let (mut wr, mut wi) = (zeros::<T>(n), zeros::<T>(n));
    f(
        false,
        false,
        n,
        &mut a,
        n,
        &mut wr,
        &mut wi,
        &mut [],
        1,
        &mut [],
        1,
    )
    .unwrap();
    let lams: Vec<C> = (0..n).map(|j| C(re(wr[j]), re(wi[j]))).collect();
    same_set("geev values-only", &lams, &real_block_eigs(), T::TOL);
}
fn check_geev_cplx<T: Num>(
    f: impl Fn(
        bool,
        bool,
        usize,
        &mut [T],
        usize,
        &mut [T],
        &mut [T],
        usize,
        &mut [T],
        usize,
    ) -> Result<()>,
) {
    let n = 3;
    let eigs = vec![C(1.0, 1.0), C(2.0, 0.0), C(-1.0, 2.0)];
    let mut t = vec![ZERO; 9];
    let off = [C(2.0, 0.0), C(1.0, -1.0), C(1.0, 0.5)];
    for i in 0..3 {
        t[i * 3 + i] = eigs[i];
    }
    t[1] = off[0];
    t[2] = off[1];
    t[5] = off[2];
    let a0 = similar_to(&t, n, 4, true);
    let mut a = tv::<T>(&a0);
    let (mut w, mut vl, mut vr) = (zeros::<T>(n), zeros::<T>(n * n), zeros::<T>(n * n));
    f(true, true, n, &mut a, n, &mut w, &mut vl, n, &mut vr, n).unwrap();
    let lams = cv(&w);
    same_set("geev eigenvalues", &lams, &eigs, T::TOL);
    let cols = |v: &[T]| -> Vec<Vec<C>> { (0..n).map(|j| col(&cv(v), n, n, j)).collect() };
    check_eigpairs(&a0, n, &lams, &cols(&vr), false, T::TOL * 5.0);
    check_eigpairs(&a0, n, &lams, &cols(&vl), true, T::TOL * 5.0);
}
#[test]
fn t_geev() {
    check_geev_real::<f64>(dgeev);
    check_geev_real::<f32>(sgeev);
    check_geev_cplx::<Complex64>(zgeev);
    check_geev_cplx::<Complex32>(cgeev);
}

fn check_gees<T: Num>(
    f: impl Fn(bool, usize, &mut [T], usize, &mut [T], &mut [T], &mut [T], usize) -> Result<usize>,
) {
    let n = 3;
    let a0 = similar_to(&real_block_t(), n, 5, false);
    let mut a = tv::<T>(&a0);
    let (mut wr, mut wi, mut vs) = (zeros::<T>(n), zeros::<T>(n), zeros::<T>(n * n));
    let sdim = f(true, n, &mut a, n, &mut wr, &mut wi, &mut vs, n).unwrap();
    assert_eq!(sdim, 0, "no ordering requested: sdim is 0");
    let lams: Vec<C> = (0..n).map(|j| C(re(wr[j]), re(wi[j]))).collect();
    same_set("gees eigenvalues", &lams, &real_block_eigs(), T::TOL);
    let (t, z) = (cv(&a), cv(&vs));
    close(
        "Z^T Z = I",
        &mm(&adj(&z, n, n), n, n, &z, n),
        &eye(n),
        T::TOL,
    );
    close(
        "Z T Z^T = A",
        &mm(&mm(&z, n, n, &t, n), n, n, &adj(&z, n, n), n),
        &a0,
        T::TOL * 5.0,
    );
    // real Schur form: quasi upper triangular (one 2x2 block)
    assert!(t[6].abs() <= T::TOL, "T[2][0] = {:?}", t[6]);
    assert!(
        t[3].abs() <= T::TOL || t[7].abs() <= T::TOL,
        "two sub-diagonal entries non-zero"
    );
}
#[test]
fn t_gees_real() {
    check_gees::<f64>(dgees);
    check_gees::<f32>(sgees);
}

/// Pair `(P Ta Q, P Tb Q)` with triangular `Ta`, `Tb`: the generalized eigenvalues are `ta_ii / tb_ii` exactly.
fn gen_pair(ta: &[C], tb: &[C], n: usize, cplx: bool) -> (Vec<C>, Vec<C>) {
    let (p, q) = (hh(n, 2, cplx), hh(n, 6, cplx));
    (
        mm(&mm(&p, n, n, ta, n), n, n, &q, n),
        mm(&mm(&p, n, n, tb, n), n, n, &q, n),
    )
}
fn tri_from(diag: [C; 3], upper: [C; 3]) -> Vec<C> {
    let mut t = vec![ZERO; 9];
    for i in 0..3 {
        t[i * 3 + i] = diag[i];
    }
    t[1] = upper[0];
    t[2] = upper[1];
    t[5] = upper[2];
    t
}
/// For every pair: right `(beta A - alpha B) v = 0`, left `u^H (beta A - alpha B) = 0`.
fn check_gen_pairs(
    a: &[C],
    b: &[C],
    n: usize,
    alpha: &[C],
    beta: &[C],
    vr: &[Vec<C>],
    vl: &[Vec<C>],
    tol: f64,
) {
    for j in 0..n {
        let m: Vec<C> = a
            .iter()
            .zip(b)
            .map(|(x, y)| beta[j] * *x - alpha[j] * *y)
            .collect();
        let rr_ = mm(&m, n, n, &vr[j], 1);
        assert!(
            vec_norm(&rr_) <= tol,
            "right residual {} at {j}",
            vec_norm(&rr_)
        );
        let ll = mm(&adj(&m, n, n), n, n, &vl[j], 1);
        assert!(
            vec_norm(&ll) <= tol,
            "left residual {} at {j}",
            vec_norm(&ll)
        );
    }
}
fn check_ggev_real<T: Num>(
    f: impl Fn(
        bool,
        bool,
        usize,
        &mut [T],
        usize,
        &mut [T],
        usize,
        &mut [T],
        &mut [T],
        &mut [T],
        &mut [T],
        usize,
        &mut [T],
        usize,
    ) -> Result<()>,
) {
    let n = 3;
    let r = |x: f64| C(x, 0.0);
    // eigenvalues 2, 1, 2.5
    let ta = tri_from([r(2.0), r(3.0), r(5.0)], [r(1.0), r(-1.0), r(0.5)]);
    let tb = tri_from([r(1.0), r(3.0), r(2.0)], [r(0.5), r(0.0), r(0.25)]);
    let (a0, b0) = gen_pair(&ta, &tb, n, false);
    let (mut a, mut b) = (tv::<T>(&a0), tv::<T>(&b0));
    let (mut ar, mut ai, mut be) = (zeros::<T>(n), zeros::<T>(n), zeros::<T>(n));
    let (mut vl, mut vr) = (zeros::<T>(n * n), zeros::<T>(n * n));
    f(
        true, true, n, &mut a, n, &mut b, n, &mut ar, &mut ai, &mut be, &mut vl, n, &mut vr, n,
    )
    .unwrap();
    let alpha: Vec<C> = (0..n).map(|j| C(re(ar[j]), re(ai[j]))).collect();
    let beta: Vec<C> = be.iter().map(|x| C(re(*x), 0.0)).collect();
    let lams: Vec<C> = (0..n).map(|j| alpha[j].div(beta[j])).collect();
    same_set("ggev eigenvalues", &lams, &[r(2.0), r(1.0), r(2.5)], T::TOL);
    let cols = |v: &[T]| -> Vec<Vec<C>> { (0..n).map(|j| col(&cv(v), n, n, j)).collect() };
    check_gen_pairs(
        &a0,
        &b0,
        n,
        &alpha,
        &beta,
        &cols(&vr),
        &cols(&vl),
        T::TOL * 10.0,
    );
}
fn check_ggev_cplx<T: Num>(
    f: impl Fn(
        bool,
        bool,
        usize,
        &mut [T],
        usize,
        &mut [T],
        usize,
        &mut [T],
        &mut [T],
        &mut [T],
        usize,
        &mut [T],
        usize,
    ) -> Result<()>,
) {
    let n = 3;
    let ta = tri_from(
        [C(2.0, 1.0), C(3.0, 0.0), C(-1.0, 2.0)],
        [C(1.0, 0.5), C(0.0, 1.0), C(0.5, 0.0)],
    );
    let tb = tri_from(
        [C(1.0, 0.0), C(1.5, 0.0), C(0.0, 2.0)],
        [C(0.5, 0.0), C(1.0, -1.0), C(0.25, 0.0)],
    );
    let want: Vec<C> = (0..3).map(|i| ta[i * 3 + i].div(tb[i * 3 + i])).collect();
    let (a0, b0) = gen_pair(&ta, &tb, n, true);
    let (mut a, mut b) = (tv::<T>(&a0), tv::<T>(&b0));
    let (mut al, mut be) = (zeros::<T>(n), zeros::<T>(n));
    let (mut vl, mut vr) = (zeros::<T>(n * n), zeros::<T>(n * n));
    f(
        true, true, n, &mut a, n, &mut b, n, &mut al, &mut be, &mut vl, n, &mut vr, n,
    )
    .unwrap();
    let (alpha, beta) = (cv(&al), cv(&be));
    let lams: Vec<C> = (0..n).map(|j| alpha[j].div(beta[j])).collect();
    same_set("ggev eigenvalues", &lams, &want, T::TOL * 5.0);
    let cols = |v: &[T]| -> Vec<Vec<C>> { (0..n).map(|j| col(&cv(v), n, n, j)).collect() };
    check_gen_pairs(
        &a0,
        &b0,
        n,
        &alpha,
        &beta,
        &cols(&vr),
        &cols(&vl),
        T::TOL * 10.0,
    );
}
#[test]
fn t_ggev() {
    check_ggev_real::<f64>(dggev);
    check_ggev_real::<f32>(sggev);
    check_ggev_cplx::<Complex64>(zggev);
    check_ggev_cplx::<Complex32>(cggev);
}

// ---------------------------------------------------------------------------------------------------------------------
// Symmetric / Hermitian eigenproblems: sygv/hegv, syevd/heevd, syevr/heevr
// ---------------------------------------------------------------------------------------------------------------------

fn check_sygv<T: Num>(
    f: impl Fn(Int, bool, Uplo, usize, &mut [T], usize, &mut [T], usize, &mut [T::R]) -> Result<()>,
) {
    let n = 3;
    // closed form: A = diag(1,2,3), B = 2 I  =>  A x = w B x  has w = 0.5, 1, 1.5
    let mut a = tv::<T>(&diag_m(&[C(1.0, 0.0), C(2.0, 0.0), C(3.0, 0.0)]));
    let mut b = tv::<T>(&diag_m(&[C(2.0, 0.0); 3]));
    let mut w = zeros::<T::R>(n);
    f(1, false, Uplo::Upper, n, &mut a, n, &mut b, n, &mut w).unwrap();
    for (k, want) in [0.5, 1.0, 1.5].iter().enumerate() {
        near("sygv closed form", re(w[k]), *want, T::TOL);
    }
    // general: residuals and B-orthonormality, all three problem types, both triangles
    let a0 = spec_herm(&[1.0, 2.0, 4.0], 3, T::CPLX);
    let b0 = spd(n, 2, T::CPLX);
    let binv = inv(&b0, n);
    for itype in 1..=3 {
        for uplo in [Uplo::Upper, Uplo::Lower] {
            let (mut pa, mut pb) = (a0.clone(), b0.clone());
            poison(&mut pa, n, uplo);
            poison(&mut pb, n, uplo);
            let (mut a, mut b) = (tv::<T>(&pa), tv::<T>(&pb));
            let mut w = zeros::<T::R>(n);
            f(itype, true, uplo, n, &mut a, n, &mut b, n, &mut w).unwrap();
            let wv: Vec<f64> = w.iter().map(|x| re(*x)).collect();
            assert!(
                wv.windows(2).all(|p| p[0] <= p[1]),
                "eigenvalues ascend: {wv:?}"
            );
            let x = cv(&a);
            let ax = |v: &[C]| mm(&a0, n, n, v, 1);
            let bx = |v: &[C]| mm(&b0, n, n, v, 1);
            for k in 0..n {
                let v = col(&x, n, n, k);
                let lhs = match itype {
                    1 => ax(&v),
                    2 => mm(&a0, n, n, &bx(&v), 1),
                    _ => mm(&b0, n, n, &ax(&v), 1),
                };
                let rhs: Vec<C> = match itype {
                    1 => bx(&v).iter().map(|c| c.sc(wv[k])).collect(),
                    _ => v.iter().map(|c| c.sc(wv[k])).collect(),
                };
                let d = vec_norm(&sub_m(&lhs, &rhs));
                assert!(
                    d <= T::TOL * 20.0,
                    "itype {itype} {uplo:?} residual {d} (k={k})"
                );
            }
            let xhx = |m: &[C]| mm(&mm(&adj(&x, n, n), n, n, m, n), n, n, &x, n);
            let metric = if itype == 3 { &binv } else { &b0 };
            close(
                &format!("itype {itype} {uplo:?} normalisation"),
                &xhx(metric),
                &eye(n),
                T::TOL * 20.0,
            );
        }
    }
}
#[test]
fn t_sygv() {
    check_sygv::<f64>(dsygv);
    check_sygv::<f32>(ssygv);
    check_sygv::<Complex64>(zhegv);
    check_sygv::<Complex32>(chegv);
}

const SPEC: [f64; 4] = [1.0, 2.0, 4.0, 7.0];

fn check_syevd<T: Num>(f: impl Fn(bool, Uplo, usize, &mut [T], usize, &mut [T::R]) -> Result<()>) {
    let n = 4;
    let a0 = spec_herm(&SPEC, 3, T::CPLX);
    for uplo in [Uplo::Upper, Uplo::Lower] {
        let mut pa = a0.clone();
        poison(&mut pa, n, uplo);
        let mut a = tv::<T>(&pa);
        let mut w = zeros::<T::R>(n);
        f(true, uplo, n, &mut a, n, &mut w).unwrap();
        for k in 0..n {
            near("evd eigenvalue", re(w[k]), SPEC[k], T::TOL * 10.0);
        }
        let v = cv(&a);
        close(
            "V^H V = I",
            &mm(&adj(&v, n, n), n, n, &v, n),
            &eye(n),
            T::TOL,
        );
        let lam = diag_m(&w.iter().map(|x| C(re(*x), 0.0)).collect::<Vec<_>>());
        close(
            "A V = V W",
            &mm(&a0, n, n, &v, n),
            &mm(&v, n, n, &lam, n),
            T::TOL * 10.0,
        );
        // values only
        let mut pa = a0.clone();
        poison(&mut pa, n, uplo);
        let mut a = tv::<T>(&pa);
        let mut w = zeros::<T::R>(n);
        f(false, uplo, n, &mut a, n, &mut w).unwrap();
        for k in 0..n {
            near("evd values-only", re(w[k]), SPEC[k], T::TOL * 10.0);
        }
    }
}
#[test]
fn t_syevd() {
    check_syevd::<f64>(dsyevd);
    check_syevd::<f32>(ssyevd);
    check_syevd::<Complex64>(zheevd);
    check_syevd::<Complex32>(cheevd);
}

fn check_syevr<T: Num>(
    f: impl Fn(
        bool,
        Range,
        Uplo,
        usize,
        &mut [T],
        usize,
        T::R,
        T::R,
        Int,
        Int,
        T::R,
        &mut [T::R],
        &mut [T],
        usize,
        &mut [Int],
    ) -> Result<usize>,
) {
    let n = 4;
    let a0 = spec_herm(&SPEC, 4, T::CPLX);
    // (range, vl, vu, il, iu, expected eigenvalues)
    let cases: [(Range, f64, f64, Int, Int, &[f64]); 3] = [
        (Range::All, 0.0, 0.0, 0, 0, &[1.0, 2.0, 4.0, 7.0]),
        (Range::Values, 1.5, 5.0, 0, 0, &[2.0, 4.0]),
        (Range::Indices, 0.0, 0.0, 2, 3, &[2.0, 4.0]),
    ];
    for uplo in [Uplo::Upper, Uplo::Lower] {
        for (range, vl, vu, il, iu, want) in cases {
            let mut pa = a0.clone();
            poison(&mut pa, n, uplo);
            let mut a = tv::<T>(&pa);
            let (mut w, mut z) = (zeros::<T::R>(n), zeros::<T>(n * n));
            let mut isuppz = vec![0 as Int; 2 * n];
            let m = f(
                true,
                range,
                uplo,
                n,
                &mut a,
                n,
                rr::<T>(vl),
                rr::<T>(vu),
                il,
                iu,
                rr::<T>(0.0),
                &mut w,
                &mut z,
                n,
                &mut isuppz,
            )
            .unwrap();
            assert_eq!(m, want.len(), "{range:?} {uplo:?}: eigenvalue count");
            let zc = cv(&z);
            for k in 0..m {
                near("evr eigenvalue", re(w[k]), want[k], T::TOL * 10.0);
                let v = col(&zc, n, n, k);
                let r = sub_m(
                    &mm(&a0, n, n, &v, 1),
                    &v.iter().map(|c| c.sc(want[k])).collect::<Vec<_>>(),
                );
                assert!(
                    vec_norm(&r) <= T::TOL * 10.0,
                    "{range:?} {uplo:?} residual {}",
                    vec_norm(&r)
                );
                near("evr eigenvector norm", vec_norm(&v), 1.0, T::TOL);
            }
        }
    }
}
#[test]
fn t_syevr() {
    check_syevr::<f64>(dsyevr);
    check_syevr::<f32>(ssyevr);
    check_syevr::<Complex64>(zheevr);
    check_syevr::<Complex32>(cheevr);
}

// ---------------------------------------------------------------------------------------------------------------------
// Partial SVD: gesvdx
// ---------------------------------------------------------------------------------------------------------------------

fn check_gesvdx<T: Num>(
    f: impl Fn(
        bool,
        bool,
        Range,
        usize,
        usize,
        &mut [T],
        usize,
        T::R,
        T::R,
        Int,
        Int,
        &mut [T::R],
        &mut [T],
        usize,
        &mut [T],
        usize,
        &mut [Int],
    ) -> Result<usize>,
) {
    let (m, n) = (4, 3);
    let sv = [5.0, 3.0, 1.0];
    // A = U S V^H with U = first n columns of a unitary, V unitary: singular values are exactly `sv`.
    let hu = hh(m, 2, T::CPLX);
    let u_thin: Vec<C> = (0..m)
        .flat_map(|i| (0..n).map(move |j| (i, j)))
        .map(|(i, j)| hu[i * m + j])
        .collect();
    let s_diag = diag_m(&sv.iter().map(|x| C(*x, 0.0)).collect::<Vec<_>>());
    let a0 = mm(&mm(&u_thin, m, n, &s_diag, n), m, n, &hh(n, 4, T::CPLX), n);
    let cases: [(Range, f64, f64, Int, Int, usize); 3] = [
        (Range::All, 0.0, 0.0, 0, 0, 3),
        (Range::Indices, 0.0, 0.0, 1, 2, 2),
        (Range::Values, 2.5, 10.0, 0, 0, 2),
    ];
    for (range, vl, vu, il, iu, want_ns) in cases {
        let mut a = tv::<T>(&a0);
        let (mut s, mut u, mut vt) = (zeros::<T::R>(n), zeros::<T>(m * n), zeros::<T>(n * n));
        let mut superb = vec![0 as Int; 12 * n];
        let ns = f(
            true,
            true,
            range,
            m,
            n,
            &mut a,
            n,
            rr::<T>(vl),
            rr::<T>(vu),
            il,
            iu,
            &mut s,
            &mut u,
            n,
            &mut vt,
            n,
            &mut superb,
        )
        .unwrap();
        assert_eq!(ns, want_ns, "{range:?}: singular value count");
        let (uc, vtc) = (cv(&u), cv(&vt));
        for k in 0..ns {
            near("singular value", re(s[k]), sv[k], T::TOL * 10.0);
            // A v_k = s_k u_k, where v_k is row k of V^H, conjugated
            let vk: Vec<C> = (0..n).map(|j| vtc[k * n + j].conj()).collect();
            let uk = col(&uc, m, n, k);
            let r = sub_m(
                &mm(&a0, m, n, &vk, 1),
                &uk.iter().map(|c| c.sc(sv[k])).collect::<Vec<_>>(),
            );
            assert!(
                vec_norm(&r) <= T::TOL * 10.0,
                "{range:?} A v = s u residual {} at {k}",
                vec_norm(&r)
            );
            near("|u_k|", vec_norm(&uk), 1.0, T::TOL);
            near("|v_k|", vec_norm(&vk), 1.0, T::TOL);
        }
    }
}
#[test]
fn t_gesvdx() {
    check_gesvdx::<f64>(dgesvdx);
    check_gesvdx::<f32>(sgesvdx);
    check_gesvdx::<Complex64>(zgesvdx);
    check_gesvdx::<Complex32>(cgesvdx);
}

// ---------------------------------------------------------------------------------------------------------------------
// QR with column pivoting
// ---------------------------------------------------------------------------------------------------------------------

fn check_geqp3<T: Num + Scalar>(
    f: impl Fn(usize, usize, &mut [T], usize, &mut [Int], &mut [T]) -> Result<()>,
    orgqr: impl Fn(usize, usize, usize, &mut [T], usize, &[T]) -> Result<()>,
) {
    let (m, n) = (4, 3);
    // closed form: columns 1*e1, 5*e2, 3*e3: pivot order must be 2, 3, 1 (1-based) with |R_ii| = 5, 3, 1
    let mut dm = vec![ZERO; m * n];
    dm[0] = C(1.0, 0.0);
    dm[n + 1] = C(5.0, 0.0);
    dm[2 * n + 2] = C(3.0, 0.0);
    let mut a = tv::<T>(&dm);
    let (mut jpvt, mut tau) = (vec![0 as Int; n], zeros::<T>(n));
    f(m, n, &mut a, n, &mut jpvt, &mut tau).unwrap();
    assert_eq!(jpvt, vec![2 as Int, 3, 1]);
    let ac = cv(&a);
    for (i, want) in [5.0, 3.0, 1.0].iter().enumerate() {
        near("|R_ii|", ac[i * n + i].abs(), *want, T::TOL);
    }
    // general matrix: Q R = A P, jpvt a permutation, |R_ii| non-increasing
    let a0 = gen(m, n, 6, T::CPLX);
    let mut a = tv::<T>(&a0);
    let (mut jpvt, mut tau) = (vec![0 as Int; n], zeros::<T>(n));
    f(m, n, &mut a, n, &mut jpvt, &mut tau).unwrap();
    let mut sorted = jpvt.clone();
    sorted.sort();
    assert_eq!(sorted, vec![1 as Int, 2, 3]);
    let r = upper_part(&cv(&a)[..n * n], n, n);
    let mut q = a.clone();
    orgqr(m, n, n, &mut q, n, &tau).unwrap();
    let mut ap = vec![ZERO; m * n];
    for j in 0..n {
        for i in 0..m {
            ap[i * n + j] = a0[i * n + (jpvt[j] as usize - 1)];
        }
    }
    close("Q R = A P", &mm(&cv(&q), m, n, &r, n), &ap, T::TOL);
    assert!(r[0].abs() + T::TOL >= r[n + 1].abs() && r[n + 1].abs() + T::TOL >= r[2 * n + 2].abs());
}
#[test]
fn t_geqp3() {
    check_geqp3::<f64>(dgeqp3, dorgqr);
    check_geqp3::<f32>(sgeqp3, sorgqr);
    check_geqp3::<Complex64>(zgeqp3, zungqr);
    check_geqp3::<Complex32>(cgeqp3, cungqr);
}

// ---------------------------------------------------------------------------------------------------------------------
// lacpy, lange, larfg, lasrt
// ---------------------------------------------------------------------------------------------------------------------

fn check_lacpy<T: Num>(
    f: impl Fn(Uplo, usize, usize, &[T], usize, &mut [T], usize),
    opposite_triangle: bool,
) {
    let (m, n, lda, ldb) = (3, 4, 5, 6);
    let a0 = gen(m, n, 1, T::CPLX);
    let sentinel = C(77.0, 0.0);
    for uplo in [Uplo::Upper, Uplo::Lower] {
        let a = tv::<T>(&pad(&a0, m, n, lda, C(1e3, 0.0)));
        let mut b = tv::<T>(&vec![sentinel; m * ldb]);
        f(uplo, m, n, &a, lda, &mut b, ldb);
        let b = cv(&b);
        for i in 0..m {
            for j in 0..ldb {
                let in_tri = match uplo {
                    Uplo::Upper => j >= i,
                    Uplo::Lower => j <= i,
                };
                // the stored triangle is copied; the padding columns (j >= n) are never touched; the opposite triangle is
                // "not referenced" per the LAPACK contract (checked only when `opposite_triangle`)
                let want = if j < n && in_tri {
                    a0[i * n + j]
                } else {
                    sentinel
                };
                if j >= n || in_tri || opposite_triangle {
                    assert!(
                        (b[i * ldb + j] - want).abs() <= 1e-6,
                        "lacpy {uplo:?} ({i},{j}): {:?} != {want:?}",
                        b[i * ldb + j]
                    );
                }
            }
        }
    }
}
#[test]
fn t_lacpy() {
    check_lacpy::<f64>(dlacpy, false);
    check_lacpy::<f32>(slacpy, false);
    check_lacpy::<Complex64>(zlacpy, false);
    check_lacpy::<Complex32>(clacpy, false);
}

fn check_lange<T: Num>(f: impl Fn(Norm, usize, usize, &[T], usize) -> T::R) {
    let (m, n, lda) = (2, 3, 4);
    let base = [1.0, -2.0, 3.0, -4.0, 5.0, -6.0];
    let a0: Vec<C> = base
        .iter()
        .enumerate()
        .map(|(k, &x)| C(x, if T::CPLX { (k % 3) as f64 - 1.0 } else { 0.0 }))
        .collect();
    let a = tv::<T>(&pad(&a0, m, n, lda, C(1e4, 1e4)));
    for norm in [Norm::Max, Norm::One, Norm::Inf, Norm::Frobenius] {
        near(
            &format!("lange {norm:?}"),
            re(f(norm, m, n, &a, lda)),
            ref_norm(&a0, m, n, norm),
            T::TOL,
        );
    }
    if !T::CPLX {
        // hand-computed: max 6, 1-norm 9 (col 3), inf-norm 15 (row 2), Frobenius sqrt(91)
        near("max", re(f(Norm::Max, m, n, &a, lda)), 6.0, T::TOL);
        near("one", re(f(Norm::One, m, n, &a, lda)), 9.0, T::TOL);
        near("inf", re(f(Norm::Inf, m, n, &a, lda)), 15.0, T::TOL);
        near(
            "fro",
            re(f(Norm::Frobenius, m, n, &a, lda)),
            91.0_f64.sqrt(),
            T::TOL,
        );
    }
}
#[test]
fn t_lange() {
    check_lange::<f64>(dlange);
    check_lange::<f32>(slange);
    check_lange::<Complex64>(zlange);
    check_lange::<Complex32>(clange);
}

fn check_larfg<T: Num>(f: impl Fn(usize, &mut T, &mut [T], Int) -> Result<T>) {
    let n = 3;
    let orig = [
        C(3.0, if T::CPLX { 1.0 } else { 0.0 }),
        C(-1.0, if T::CPLX { 2.0 } else { 0.0 }),
        C(2.0, 0.0),
    ];
    let mut alpha = T::mk(orig[0].0, orig[0].1);
    let mut x = tv::<T>(&orig[1..]);
    let tau = cv(&[f(n, &mut alpha, &mut x, 1).unwrap()])[0];
    let beta = cv(&[alpha])[0];
    // H^H [alpha; x] = [beta; 0; 0] with H = I - tau v v^H, v = [1; x']
    let mut v = vec![ONE];
    v.extend(cv(&x));
    let vvh = mm(&v, n, 1, &adj(&v, n, 1), n);
    let mut h = eye(n);
    for k in 0..n * n {
        h[k] = h[k] - tau * vvh[k];
    }
    let out = mm(&adj(&h, n, n), n, n, &orig, 1);
    close("H^H x = beta e1", &out, &[beta, ZERO, ZERO], T::TOL);
    near("|beta| = ||x||", beta.abs(), vec_norm(&orig), T::TOL);
    near("beta is real", beta.1, 0.0, T::TOL);
    // beta takes the sign opposite to Re(alpha)
    assert!(beta.0 < 0.0, "beta = {beta:?}");
}
#[test]
fn t_larfg() {
    check_larfg::<f64>(dlarfg);
    check_larfg::<f32>(slarfg);
    check_larfg::<Complex64>(zlarfg);
    check_larfg::<Complex32>(clarfg);
}

fn check_lasrt<T: Num>(f: impl Fn(u8, &mut [T]) -> Result<()>) {
    let data: [f64; 8] = [3.0, -1.0, 4.0, 1.0, -5.0, 9.0, 2.0, 6.0];
    let mut inc = data.to_vec();
    inc.sort_by(|a, b| a.total_cmp(b));
    let mut dec = inc.clone();
    dec.reverse();
    for (id, want) in [(b'I', &inc), (b'D', &dec)] {
        let mut d: Vec<T> = data.iter().map(|x| T::mk(*x, 0.0)).collect();
        f(id, &mut d).unwrap();
        assert_eq!(
            d.iter().map(|x| re(*x)).collect::<Vec<_>>(),
            *want,
            "id {}",
            id as char
        );
    }
    let mut d: Vec<T> = data.iter().map(|x| T::mk(*x, 0.0)).collect();
    assert!(
        f(b'X', &mut d).is_err(),
        "an unknown sort id is an illegal argument"
    );
}
#[test]
fn t_lasrt() {
    check_lasrt::<f64>(dlasrt);
    check_lasrt::<f32>(slasrt);
}

// ---------------------------------------------------------------------------------------------------------------------
// Symmetric/Hermitian indefinite factorisations: sytrf/sytrs/sytri, hetrf/hetrs/hetri, hesv
// ---------------------------------------------------------------------------------------------------------------------

/// Indefinite (mixed-sign diagonal) matrix; Hermitian when `herm`, else (complex-)symmetric.
fn indefinite(n: usize, herm: bool, cplx: bool) -> Vec<C> {
    let mut a = vec![ZERO; n * n];
    for i in 0..n {
        let d = 1.0 + 0.5 * i as f64;
        a[i * n + i] = C(if i % 2 == 0 { d } else { -d }, 0.0);
        for j in i + 1..n {
            let re_ = (((i + 2 * j) % 5) as f64) / 2.0 + 0.5;
            let im_ = if cplx {
                (((2 * i + j) % 4) as f64) / 2.0 - 0.5
            } else {
                0.0
            };
            a[i * n + j] = C(re_, im_);
            a[j * n + i] = if herm { C(re_, -im_) } else { C(re_, im_) };
        }
    }
    a
}
fn check_sytr<T: Num>(
    herm: bool,
    trf: impl Fn(Uplo, usize, &mut [T], usize, &mut [Int]) -> Result<()>,
    trs: impl Fn(Uplo, usize, usize, &[T], usize, &[Int], &mut [T], usize) -> Result<()>,
    tri: impl Fn(Uplo, usize, &mut [T], usize, &[Int]) -> Result<()>,
) {
    let (n, nrhs) = (4, 2);
    let a0 = indefinite(n, herm, T::CPLX);
    let x = gen(n, nrhs, 3, T::CPLX);
    let b = mm(&a0, n, n, &x, nrhs);
    for uplo in [Uplo::Upper, Uplo::Lower] {
        let mut pa = a0.clone();
        poison(&mut pa, n, uplo);
        let mut a = tv::<T>(&pa);
        let mut ipiv = vec![0 as Int; n];
        trf(uplo, n, &mut a, n, &mut ipiv).unwrap();
        let mut bt = tv::<T>(&b);
        trs(uplo, n, nrhs, &a, n, &ipiv, &mut bt, nrhs).unwrap();
        close(
            &format!("trs {uplo:?}: solution x"),
            &cv(&bt),
            &x,
            T::TOL * 20.0,
        );
        tri(uplo, n, &mut a, n, &ipiv).unwrap();
        let invf = mirror(&cv(&a), n, uplo, herm);
        close(
            &format!("tri {uplo:?}: A inv(A) = I"),
            &mm(&a0, n, n, &invf, n),
            &eye(n),
            T::TOL * 20.0,
        );
    }
    // a zero matrix is singular: the factorisation reports it
    let mut z = zeros::<T>(n * n);
    let mut ipiv = vec![0 as Int; n];
    assert!(trf(Uplo::Upper, n, &mut z, n, &mut ipiv).is_err());
}
#[test]
fn t_sytr() {
    check_sytr::<f64>(false, dsytrf, dsytrs, dsytri);
    check_sytr::<f32>(false, ssytrf, ssytrs, ssytri);
    check_sytr::<Complex64>(false, zsytrf, zsytrs, zsytri);
    check_sytr::<Complex32>(false, csytrf, csytrs, csytri);
}
#[test]
fn t_hetr() {
    check_sytr::<Complex64>(true, zhetrf, zhetrs, zhetri);
    check_sytr::<Complex32>(true, chetrf, chetrs, chetri);
}

fn check_hesv<T: Num + ComplexScalar>() {
    let (n, nrhs) = (4, 2);
    let a0 = indefinite(n, true, true);
    let x = gen(n, nrhs, 5, true);
    let b = mm(&a0, n, n, &x, nrhs);
    for uplo in [Uplo::Upper, Uplo::Lower] {
        let mut pa = a0.clone();
        poison(&mut pa, n, uplo);
        let (mut a, mut bt) = (tv::<T>(&pa), tv::<T>(&b));
        let mut ipiv = vec![0 as Int; n];
        hesv::<T>(uplo, n, &mut a, &mut ipiv, &mut bt).unwrap();
        close(&format!("hesv {uplo:?}"), &cv(&bt), &x, T::TOL * 20.0);
    }
    // singular (all zero) reports the failing pivot
    let (mut a, mut bt, mut ipiv) = (zeros::<T>(4), zeros::<T>(2), vec![0 as Int; 2]);
    assert!(matches!(
        hesv::<T>(Uplo::Upper, 2, &mut a, &mut ipiv, &mut bt),
        Err(Error::Status { .. })
    ));
}
#[test]
fn t_hesv() {
    check_hesv::<Complex64>();
    check_hesv::<Complex32>();
}

// ---------------------------------------------------------------------------------------------------------------------
// gelss (SVD least squares)
// ---------------------------------------------------------------------------------------------------------------------

fn check_gelss<T: Num>(
    f: impl Fn(
        usize,
        usize,
        usize,
        &mut [T],
        usize,
        &mut [T],
        usize,
        &mut [T::R],
        T::R,
    ) -> Result<usize>,
) {
    let (m, n) = (4, 3);
    let rcond = rr::<T>(1e-4);
    // full rank, consistent: b = A x  =>  solution x, rank 3, sum s^2 = ||A||_F^2
    let a0 = gen(m, n, 2, T::CPLX);
    let x = vec![
        C(1.0, if T::CPLX { 0.5 } else { 0.0 }),
        C(-2.0, 0.0),
        C(3.0, 0.0),
    ];
    let mut b = zeros::<T>(m);
    b.copy_from_slice(&tv::<T>(&mm(&a0, m, n, &x, 1)));
    let mut a = tv::<T>(&a0);
    let mut s = zeros::<T::R>(n);
    let rank = f(m, n, 1, &mut a, n, &mut b, 1, &mut s, rcond).unwrap();
    assert_eq!(rank, 3);
    close("gelss solution", &cv(&b)[..n], &x, T::TOL * 10.0);
    let ss: f64 = s.iter().map(|v| re(*v) * re(*v)).sum();
    near(
        "sum s^2 = ||A||_F^2",
        ss,
        ref_norm(&a0, m, n, Norm::Frobenius).powi(2),
        T::TOL * 100.0,
    );
    // rank 2: third column = first + second; b = c1 + 2 c2 (= A (1,2,0)); the minimum-norm solution is (0,1,1)
    let mut ad = a0.clone();
    for i in 0..m {
        ad[i * n + 2] = ad[i * n] + ad[i * n + 1];
    }
    let xd = [C(1.0, 0.0), C(2.0, 0.0), ZERO];
    let mut b = zeros::<T>(m);
    b.copy_from_slice(&tv::<T>(&mm(&ad, m, n, &xd, 1)));
    let mut a = tv::<T>(&ad);
    let mut s = zeros::<T::R>(n);
    let rank = f(m, n, 1, &mut a, n, &mut b, 1, &mut s, rcond).unwrap();
    assert_eq!(rank, 2);
    close(
        "gelss min-norm solution",
        &cv(&b)[..n],
        &[ZERO, ONE, ONE],
        T::TOL * 10.0,
    );
}
#[test]
fn t_gelss() {
    check_gelss::<f64>(dgelss);
    check_gelss::<f32>(sgelss);
    check_gelss::<Complex64>(zgelss);
    check_gelss::<Complex32>(cgelss);
}

// ---------------------------------------------------------------------------------------------------------------------
// Reductions: gehrd, gebrd (+ orgbr/ungbr), orgtr/ungtr
// ---------------------------------------------------------------------------------------------------------------------

fn check_gehrd<T: Num + Scalar>(
    f: impl Fn(usize, Int, Int, &mut [T], usize, &mut [T]) -> Result<()>,
    orgqr: impl Fn(usize, usize, usize, &mut [T], usize, &[T]) -> Result<()>,
) {
    let n = 4;
    let a0 = gen(n, n, 3, T::CPLX);
    let mut a = tv::<T>(&a0);
    let mut tau = zeros::<T>(n - 1);
    f(n, 1, n as Int, &mut a, n, &mut tau).unwrap();
    let ac = cv(&a);
    // H = upper Hessenberg part
    let mut h = ac.clone();
    for i in 2..n {
        for j in 0..i - 1 {
            h[i * n + j] = ZERO;
        }
    }
    // invariants of a unitary similarity
    let tr = |m: &[C]| (0..n).fold(ZERO, |s, i| s + m[i * n + i]);
    close("trace(H) = trace(A)", &[tr(&h)], &[tr(&a0)], T::TOL * 10.0);
    near(
        "||H||_F = ||A||_F",
        ref_norm(&h, n, n, Norm::Frobenius),
        ref_norm(&a0, n, n, Norm::Frobenius),
        T::TOL * 10.0,
    );
    // Q = diag(1, Q'), Q' = orgqr on the (n-1)x(n-1) reflector block below the sub-diagonal
    let k = n - 1;
    let mut sub = zeros::<T>(k * k);
    for i in 0..k {
        for j in 0..k {
            sub[i * k + j] = a[(i + 1) * n + j];
        }
    }
    orgqr(k, k, k, &mut sub, k, &tau).unwrap();
    let mut q = eye(n);
    for i in 0..k {
        for j in 0..k {
            q[(i + 1) * n + j + 1] = cv(&sub)[i * k + j];
        }
    }
    close(
        "Q^H Q = I",
        &mm(&adj(&q, n, n), n, n, &q, n),
        &eye(n),
        T::TOL,
    );
    close(
        "Q H Q^H = A",
        &mm(&mm(&q, n, n, &h, n), n, n, &adj(&q, n, n), n),
        &a0,
        T::TOL * 10.0,
    );
}
#[test]
fn t_gehrd() {
    check_gehrd::<f64>(dgehrd, dorgqr);
    check_gehrd::<f32>(sgehrd, sorgqr);
    check_gehrd::<Complex64>(zgehrd, zungqr);
    check_gehrd::<Complex32>(cgehrd, cungqr);
}

fn check_gebrd<T: Num>(
    gebrd: impl Fn(
        usize,
        usize,
        &mut [T],
        usize,
        &mut [T::R],
        &mut [T::R],
        &mut [T],
        &mut [T],
    ) -> Result<()>,
    orgbr: impl Fn(BidiagVect, usize, usize, usize, &mut [T], usize, &[T]) -> Result<()>,
) {
    let (m, n) = (4, 3);
    let a0 = gen(m, n, 4, T::CPLX);
    let mut a = tv::<T>(&a0);
    let (mut d, mut e) = (zeros::<T::R>(n), zeros::<T::R>(n - 1));
    let (mut tauq, mut taup) = (zeros::<T>(n), zeros::<T>(n));
    gebrd(m, n, &mut a, n, &mut d, &mut e, &mut tauq, &mut taup).unwrap();
    // B: upper bidiagonal n x n
    let mut b = vec![ZERO; n * n];
    for i in 0..n {
        b[i * n + i] = C(re(d[i]), 0.0);
        if i + 1 < n {
            b[i * n + i + 1] = C(re(e[i]), 0.0);
        }
    }
    near(
        "||B||_F = ||A||_F",
        ref_norm(&b, n, n, Norm::Frobenius),
        ref_norm(&a0, m, n, Norm::Frobenius),
        T::TOL * 10.0,
    );
    let mut q = a.clone();
    orgbr(BidiagVect::Q, m, n, n, &mut q, n, &tauq).unwrap();
    let mut pt = a[..n * n].to_vec();
    orgbr(BidiagVect::P, n, n, n, &mut pt, n, &taup).unwrap();
    let (qc, ptc) = (cv(&q), cv(&pt));
    close(
        "Q^H Q = I",
        &mm(&adj(&qc, m, n), n, m, &qc, n),
        &eye(n),
        T::TOL,
    );
    close(
        "P^H P = I",
        &mm(&ptc, n, n, &adj(&ptc, n, n), n),
        &eye(n),
        T::TOL,
    );
    close(
        "Q B P^H = A",
        &mm(&mm(&qc, m, n, &b, n), m, n, &ptc, n),
        &a0,
        T::TOL * 10.0,
    );
}
#[test]
fn t_gebrd_orgbr() {
    check_gebrd::<f64>(dgebrd, dorgbr);
    check_gebrd::<f32>(sgebrd, sorgbr);
    check_gebrd::<Complex64>(zgebrd, zungbr);
    check_gebrd::<Complex32>(cgebrd, cungbr);
}

/// Hand-built reflectors `H_k = I - tau v v^H` (`tau = 2 / v^H v`, so `H_k` is unitary) stored the way sytrd stores them;
/// the expected `Q` is their explicit product, computed here.
fn check_orgtr<T: Num>(f: impl Fn(Uplo, usize, &mut [T], usize, &[T]) -> Result<()>) {
    let n = 4;
    for uplo in [Uplo::Upper, Uplo::Lower] {
        let mut stored = vec![C(55.0, 0.0); n * n]; // garbage where no reflector lives
        let mut tau = vec![ZERO; n - 1];
        let mut hs: Vec<Vec<C>> = Vec::new();
        for k in 0..n - 1 {
            let w = gen(n, 1, 3 + k, T::CPLX);
            let mut v = vec![ZERO; n];
            match uplo {
                // Upper: v(k) = 1, v(0..k) stored in column k+1; Lower: v(k+1) = 1, v(k+2..) stored in column k
                Uplo::Upper => {
                    v[k] = ONE;
                    for r in 0..k {
                        v[r] = w[r];
                        stored[r * n + k + 1] = w[r];
                    }
                }
                Uplo::Lower => {
                    v[k + 1] = ONE;
                    for r in k + 2..n {
                        v[r] = w[r];
                        stored[r * n + k] = w[r];
                    }
                }
            }
            let nrm: f64 = v.iter().map(|x| x.abs() * x.abs()).sum();
            tau[k] = C(2.0 / nrm, 0.0);
            let vvh = mm(&v, n, 1, &adj(&v, n, 1), n);
            let mut h = eye(n);
            for i in 0..n * n {
                h[i] = h[i] - vvh[i].sc(2.0 / nrm);
            }
            hs.push(h);
        }
        // Upper: Q = H_{n-2} ... H_0;  Lower: Q = H_0 ... H_{n-2}
        let mut q = eye(n);
        for k in 0..n - 1 {
            let h = &hs[k];
            q = match uplo {
                Uplo::Upper => mm(h, n, n, &q, n),
                Uplo::Lower => mm(&q, n, n, h, n),
            };
        }
        let mut a = tv::<T>(&stored);
        f(uplo, n, &mut a, n, &tv::<T>(&tau)).unwrap();
        close(&format!("orgtr {uplo:?}"), &cv(&a), &q, T::TOL * 5.0);
    }
}
#[test]
fn t_orgtr() {
    check_orgtr::<f64>(dorgtr);
    check_orgtr::<f32>(sorgtr);
    check_orgtr::<Complex64>(zungtr);
    check_orgtr::<Complex32>(cungtr);
}

// ---------------------------------------------------------------------------------------------------------------------
// Condition estimates: gecon, pocon
// ---------------------------------------------------------------------------------------------------------------------

fn check_gecon<T: Num + Scalar>(f: impl Fn(Norm, usize, &[T], usize, T::R) -> Result<T::R>) {
    let n = 3;
    // closed form: diag(1,2,4) has kappa_1 = kappa_inf = 4, so rcond = 0.25 (the estimator is exact for diagonals)
    let dm = diag_m(&[C(1.0, 0.0), C(2.0, 0.0), C(4.0, 0.0)]);
    for norm in [Norm::One, Norm::Inf] {
        let mut a = tv::<T>(&dm);
        let mut ipiv = vec![0 as Int; n];
        getrf::<T>(n, n, &mut a, &mut ipiv).unwrap();
        let rc = f(norm, n, &a, n, rr::<T>(4.0)).unwrap();
        near(&format!("gecon diag {norm:?}"), re(rc), 0.25, T::TOL);
    }
    // general: the estimate is a lower bound on ||A^-1||, so rcond_est >= true rcond (and within a small factor)
    let a0 = gen(n, n, 2, T::CPLX);
    let ai = inv(&a0, n);
    for norm in [Norm::One, Norm::Inf] {
        let truth = 1.0 / (ref_norm(&a0, n, n, norm) * ref_norm(&ai, n, n, norm));
        let mut a = tv::<T>(&a0);
        let mut ipiv = vec![0 as Int; n];
        getrf::<T>(n, n, &mut a, &mut ipiv).unwrap();
        let rc = re(f(norm, n, &a, n, rr::<T>(ref_norm(&a0, n, n, norm))).unwrap());
        assert!(
            rc >= truth * (1.0 - 1e-3) && rc <= truth * 3.0,
            "gecon {norm:?}: estimate {rc}, true {truth}"
        );
    }
}
#[test]
fn t_gecon() {
    check_gecon::<f64>(dgecon);
    check_gecon::<f32>(sgecon);
    check_gecon::<Complex64>(zgecon);
    check_gecon::<Complex32>(cgecon);
}

fn check_pocon<T: Num + Scalar>(f: impl Fn(Uplo, usize, &[T], usize, T::R) -> Result<T::R>) {
    let n = 3;
    let dm = diag_m(&[C(1.0, 0.0), C(2.0, 0.0), C(4.0, 0.0)]);
    for uplo in [Uplo::Upper, Uplo::Lower] {
        let mut a = tv::<T>(&dm);
        potrf::<T>(uplo, n, &mut a).unwrap();
        let rc = f(uplo, n, &a, n, rr::<T>(4.0)).unwrap();
        near(&format!("pocon diag {uplo:?}"), re(rc), 0.25, T::TOL);
        let a0 = spd(n, 2, T::CPLX);
        let truth =
            1.0 / (ref_norm(&a0, n, n, Norm::One) * ref_norm(&inv(&a0, n), n, n, Norm::One));
        let mut a = tv::<T>(&a0);
        potrf::<T>(uplo, n, &mut a).unwrap();
        let rc = re(f(uplo, n, &a, n, rr::<T>(ref_norm(&a0, n, n, Norm::One))).unwrap());
        assert!(
            rc >= truth * (1.0 - 1e-3) && rc <= truth * 3.0,
            "pocon {uplo:?}: estimate {rc}, true {truth}"
        );
    }
}
#[test]
fn t_pocon() {
    check_pocon::<f64>(dpocon);
    check_pocon::<f32>(spocon);
    check_pocon::<Complex64>(zpocon);
    check_pocon::<Complex32>(cpocon);
}

// keep helper `scale`/`is_ident` referenced so the file builds warning-free without needing them everywhere
#[test]
fn t_reference_helpers_self_check() {
    // the reference arithmetic itself: (1+i)(1-i) = 2, inv(diag(2,4)) = diag(.5,.25), I is identity
    assert_eq!(C(1.0, 1.0) * C(1.0, -1.0), C(2.0, 0.0));
    let d = diag_m(&[C(2.0, 0.0), C(4.0, 0.0)]);
    close(
        "inv",
        &inv(&d, 2),
        &diag_m(&[C(0.5, 0.0), C(0.25, 0.0)]),
        1e-15,
    );
    assert!(is_ident(&scale(&eye(2), ONE), 2) == 0.0);
}
