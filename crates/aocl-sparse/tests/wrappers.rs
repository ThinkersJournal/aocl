//! Result-checked tests for the aocl-sparse safe wrappers that no unit test in `src/` exercises, run on the real AOCL.
//!
//! Every expected value is built HERE, from a small dense reference computed in the test (a serial dense product, a
//! residual `||op(A) x - alpha b||`, a dense reconstruction of a converted format): nothing is copied from the library's
//! output. Each generic checker runs once per exposed precision (`f32`, `f64`, `Complex32`, `Complex64`) through the
//! `Lib` trait, which carries the handle type, the tolerance and the conversion to the reference's complex scalar `Cx`.

use aocl_sparse as s;
use aocl_sparse::complex as cxm;
use aocl_sparse::{
    Complex32, Complex64, DiagType, FillMode, IndexBase, MatDescr, MatType, Order, Result, SorType,
    Trans,
};
use aocl_sparse_sys as sys;
use std::ops::{Add, Div, Mul, Sub};

type I = sys::aoclsparse_int;
type Raw = sys::aoclsparse_matrix;

// ---------------------------------------------------------------- reference arithmetic (f64 complex)

#[derive(Clone, Copy, Debug, PartialEq)]
struct Cx {
    re: f64,
    im: f64,
}
const fn c(re: f64, im: f64) -> Cx {
    Cx { re, im }
}
const ZERO: Cx = c(0.0, 0.0);
const ONE: Cx = c(1.0, 0.0);
impl Add for Cx {
    type Output = Cx;
    fn add(self, o: Cx) -> Cx {
        c(self.re + o.re, self.im + o.im)
    }
}
impl Sub for Cx {
    type Output = Cx;
    fn sub(self, o: Cx) -> Cx {
        c(self.re - o.re, self.im - o.im)
    }
}
impl Mul for Cx {
    type Output = Cx;
    fn mul(self, o: Cx) -> Cx {
        c(
            self.re * o.re - self.im * o.im,
            self.re * o.im + self.im * o.re,
        )
    }
}
impl Div for Cx {
    type Output = Cx;
    fn div(self, o: Cx) -> Cx {
        let n = o.re * o.re + o.im * o.im;
        let t = self * o.conj();
        c(t.re / n, t.im / n)
    }
}
impl Cx {
    fn conj(self) -> Cx {
        c(self.re, -self.im)
    }
    fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
}

/// One precision: the handle type, the tolerance and the conversion to/from the reference scalar.
trait Lib: Copy + std::fmt::Debug {
    type M;
    const CPLX: bool;
    const TOL: f64;
    fn mk(x: Cx) -> Self;
    fn get(self) -> Cx;
    fn build(base: IndexBase, a: &Csr) -> Self::M;
    fn own(raw: Raw) -> Result<Self::M>;
    fn raw(m: &Self::M) -> Raw;
    fn export(m: &Self::M) -> (IndexBase, Vec<I>, Vec<I>, Vec<Cx>);
    /// `aoclsparse_?trsv` on a raw handle (for the TCSR handles the safe matrix types cannot wrap).
    fn trsv_raw(
        raw: Raw,
        d: &MatDescr,
        alpha: Self,
        b: &[Self],
        x: &mut [Self],
    ) -> sys::aoclsparse_status;
}
macro_rules! lib_real {
    ($t:ty, $tol:expr, $symgs:ident) => {
        impl Lib for $t {
            type M = s::SparseMatrix<$t>;
            const CPLX: bool = false;
            const TOL: f64 = $tol;
            fn mk(x: Cx) -> Self {
                x.re as $t
            }
            fn get(self) -> Cx {
                c(self as f64, 0.0)
            }
            fn build(base: IndexBase, a: &Csr) -> Self::M {
                let (rp, ci) = a.arrays(base);
                s::SparseMatrix::<$t>::from_csr(base, a.m, a.n, &rp, &ci, &lv::<$t>(&a.v)).unwrap()
            }
            fn own(raw: Raw) -> Result<Self::M> {
                unsafe { s::SparseMatrix::<$t>::from_library_owned(raw) }
            }
            fn raw(m: &Self::M) -> Raw {
                m.as_raw()
            }
            fn export(m: &Self::M) -> (IndexBase, Vec<I>, Vec<I>, Vec<Cx>) {
                let (b, rp, ci, v) = m.export_csr().unwrap();
                (b, rp, ci, cv(&v))
            }
            fn trsv_raw(
                raw: Raw,
                d: &MatDescr,
                alpha: Self,
                b: &[Self],
                x: &mut [Self],
            ) -> sys::aoclsparse_status {
                unsafe {
                    sys::$symgs(
                        sys::aoclsparse_operation__aoclsparse_operation_none,
                        alpha,
                        raw,
                        d.as_raw(),
                        b.as_ptr(),
                        x.as_mut_ptr(),
                    )
                }
            }
        }
    };
}
macro_rules! lib_cplx {
    ($t:ty, $tol:expr, $re:ty, $symgs:ident, $cs:ident) => {
        impl Lib for $t {
            type M = cxm::ComplexSparseMatrix<$t>;
            const CPLX: bool = true;
            const TOL: f64 = $tol;
            fn mk(x: Cx) -> Self {
                <$t>::new(x.re as $re, x.im as $re)
            }
            fn get(self) -> Cx {
                c(self.re as f64, self.im as f64)
            }
            fn build(base: IndexBase, a: &Csr) -> Self::M {
                let (rp, ci) = a.arrays(base);
                cxm::ComplexSparseMatrix::<$t>::from_csr(base, a.m, a.n, &rp, &ci, &lv::<$t>(&a.v))
                    .unwrap()
            }
            fn own(raw: Raw) -> Result<Self::M> {
                unsafe { cxm::ComplexSparseMatrix::<$t>::from_library_owned(raw) }
            }
            fn raw(m: &Self::M) -> Raw {
                m.as_raw()
            }
            fn export(m: &Self::M) -> (IndexBase, Vec<I>, Vec<I>, Vec<Cx>) {
                let (b, rp, ci, v) = m.export_csr().unwrap();
                (b, rp, ci, cv(&v))
            }
            fn trsv_raw(
                raw: Raw,
                d: &MatDescr,
                alpha: Self,
                b: &[Self],
                x: &mut [Self],
            ) -> sys::aoclsparse_status {
                let al = sys::$cs {
                    real: alpha.re,
                    imag: alpha.im,
                };
                unsafe {
                    sys::$symgs(
                        sys::aoclsparse_operation__aoclsparse_operation_none,
                        al,
                        raw,
                        d.as_raw(),
                        b.as_ptr() as *const _,
                        x.as_mut_ptr() as *mut _,
                    )
                }
            }
        }
    };
}
lib_real!(f32, 2e-4, aoclsparse_strsv);
lib_real!(f64, 1e-11, aoclsparse_dtrsv);
lib_cplx!(
    Complex32,
    2e-4,
    f32,
    aoclsparse_ctrsv,
    aoclsparse_float_complex
);
lib_cplx!(
    Complex64,
    1e-11,
    f64,
    aoclsparse_ztrsv,
    aoclsparse_double_complex_
);

fn lv<T: Lib>(a: &[Cx]) -> Vec<T> {
    a.iter().map(|&x| T::mk(x)).collect()
}
fn cv<T: Lib>(a: &[T]) -> Vec<Cx> {
    a.iter().map(|x| x.get()).collect()
}
/// A scalar of the reference with the imaginary part dropped for the real precisions.
fn v<T: Lib>(re: f64, im: f64) -> Cx {
    c(re, if T::CPLX { im } else { 0.0 })
}
fn near(what: &str, k: usize, got: Cx, want: Cx, tol: f64) {
    assert!(
        (got - want).abs() <= tol * (1.0 + want.abs()),
        "{what}[{k}]: got {got:?}, want {want:?}"
    );
}
fn checkc(what: &str, got: &[Cx], want: &[Cx], tol: f64) {
    assert_eq!(got.len(), want.len(), "{what}: length");
    for (k, (g, w)) in got.iter().zip(want).enumerate() {
        near(what, k, *g, *w, tol);
    }
}
fn check<T: Lib>(what: &str, got: &[T], want: &[Cx]) {
    checkc(what, &cv(got), want, T::TOL);
}
fn ops<T: Lib>() -> Vec<Trans> {
    if T::CPLX {
        vec![Trans::No, Trans::T, Trans::C]
    } else {
        vec![Trans::No, Trans::T]
    }
}

// ---------------------------------------------------------------- dense reference + CSR building

type D = Vec<Vec<Cx>>;
fn nrow(a: &D) -> usize {
    a.len()
}
fn ncol(a: &D) -> usize {
    a.first().map_or(0, |r| r.len())
}
fn im_of<T: Lib>(i: usize, j: usize) -> f64 {
    if T::CPLX {
        ((2 * i + 3 * j) % 5) as f64 * 0.25 - 0.5
    } else {
        0.0
    }
}
/// Dense matrix from the real parts (zero stays structurally zero); complex precisions get a nonzero imaginary part.
fn dm<T: Lib>(re: &[&[f64]]) -> D {
    re.iter()
        .enumerate()
        .map(|(i, r)| {
            r.iter()
                .enumerate()
                .map(|(j, &x)| {
                    if x == 0.0 {
                        ZERO
                    } else {
                        c(x, im_of::<T>(i, j))
                    }
                })
                .collect()
        })
        .collect()
}
/// Symmetric (real) / Hermitian (complex) matrix from a symmetric real-part table: A[j][i] = conj(A[i][j]), real diagonal.
fn herm<T: Lib>(re: &[&[f64]]) -> D {
    let n = re.len();
    (0..n)
        .map(|i| {
            (0..n)
                .map(|j| match (re[i][j], i.cmp(&j)) {
                    (x, _) if x == 0.0 => ZERO,
                    (x, std::cmp::Ordering::Equal) => c(x, 0.0),
                    (x, std::cmp::Ordering::Less) => c(x, im_of::<T>(i, j)),
                    (x, std::cmp::Ordering::Greater) => c(x, -im_of::<T>(j, i)),
                })
                .collect()
        })
        .collect()
}
fn vecg<T: Lib>(n: usize, seed: usize) -> Vec<Cx> {
    (0..n)
        .map(|k| {
            v::<T>(
                1.0 + 0.5 * ((3 * k + seed) % 7) as f64,
                0.25 * ((k + seed) % 4) as f64 - 0.3,
            )
        })
        .collect()
}
fn opd(a: &D, op: Trans) -> D {
    match op {
        Trans::No => a.clone(),
        Trans::T => (0..ncol(a))
            .map(|j| (0..nrow(a)).map(|i| a[i][j]).collect())
            .collect(),
        Trans::C => (0..ncol(a))
            .map(|j| (0..nrow(a)).map(|i| a[i][j].conj()).collect())
            .collect(),
    }
}
fn mv_d(a: &D, x: &[Cx]) -> Vec<Cx> {
    a.iter()
        .map(|r| r.iter().zip(x).fold(ZERO, |s, (&p, &q)| s + p * q))
        .collect()
}
fn mm_d(a: &D, b: &D) -> D {
    (0..nrow(a))
        .map(|i| {
            (0..ncol(b))
                .map(|j| (0..ncol(a)).fold(ZERO, |s, k| s + a[i][k] * b[k][j]))
                .collect()
        })
        .collect()
}
/// Zero-based CSR of the nonzeros of a dense matrix (the sparse input the library gets).
struct Csr {
    m: usize,
    n: usize,
    rp: Vec<I>,
    ci: Vec<I>,
    v: Vec<Cx>,
}
fn csr(a: &D) -> Csr {
    let (mut rp, mut ci, mut v) = (vec![0 as I], vec![], vec![]);
    for r in a {
        for (j, &x) in r.iter().enumerate() {
            if x != ZERO {
                ci.push(j as I);
                v.push(x);
            }
        }
        rp.push(ci.len() as I);
    }
    Csr {
        m: nrow(a),
        n: ncol(a),
        rp,
        ci,
        v,
    }
}
fn off(base: IndexBase) -> I {
    (base == IndexBase::One) as I
}
impl Csr {
    fn arrays(&self, base: IndexBase) -> (Vec<I>, Vec<I>) {
        let o = off(base);
        (
            self.rp.iter().map(|p| p + o).collect(),
            self.ci.iter().map(|j| j + o).collect(),
        )
    }
}
/// Dense reconstruction of a (possibly base-one) CSR.
fn csr_dense(m: usize, n: usize, base: IndexBase, rp: &[I], ci: &[I], val: &[Cx]) -> D {
    let o = off(base);
    let mut d = vec![vec![ZERO; n]; m];
    for i in 0..m {
        for k in (rp[i] - o) as usize..(rp[i + 1] - o) as usize {
            d[i][(ci[k] - o) as usize] = d[i][(ci[k] - o) as usize] + val[k];
        }
    }
    d
}
fn dense_eq(what: &str, got: &D, want: &D, tol: f64) {
    assert_eq!(
        (nrow(got), ncol(got)),
        (nrow(want), ncol(want)),
        "{what}: dims"
    );
    for i in 0..nrow(got) {
        checkc(&format!("{what} row {i}"), &got[i], &want[i], tol);
    }
}
fn descr(ty: MatType, fill: FillMode, diag: DiagType, base: IndexBase) -> MatDescr {
    let mut d = MatDescr::new().unwrap();
    d.set_type(ty).unwrap();
    d.set_fill_mode(fill).unwrap();
    d.set_diag_type(diag).unwrap();
    d.set_index_base(base).unwrap();
    d
}
fn gen_descr(base: IndexBase) -> MatDescr {
    descr(MatType::General, FillMode::Lower, DiagType::NonUnit, base)
}
fn dense_ld(d: &D, order: Order, ld: usize) -> Vec<Cx> {
    let (m, n) = (nrow(d), ncol(d));
    let mut out = vec![
        c(-9.0, -9.0);
        if order == Order::RowMajor {
            m * ld
        } else {
            n * ld
        }
    ];
    for i in 0..m {
        for j in 0..n {
            out[if order == Order::RowMajor {
                i * ld + j
            } else {
                j * ld + i
            }] = d[i][j];
        }
    }
    out
}
fn from_ld(buf: &[Cx], m: usize, n: usize, order: Order, ld: usize) -> D {
    (0..m)
        .map(|i| {
            (0..n)
                .map(|j| {
                    buf[if order == Order::RowMajor {
                        i * ld + j
                    } else {
                        j * ld + i
                    }]
                })
                .collect()
        })
        .collect()
}
fn sym_ty<T: Lib>() -> MatType {
    if T::CPLX {
        MatType::Hermitian
    } else {
        MatType::Symmetric
    }
}

// ---------------------------------------------------------------- MatDescr, version

#[test]
fn descr_setters_and_getters_round_trip() {
    let mut d = MatDescr::new().unwrap();
    for t in [
        MatType::Symmetric,
        MatType::Hermitian,
        MatType::Triangular,
        MatType::General,
    ] {
        d.set_type(t).unwrap();
        assert_eq!(d.ty(), t);
    }
    for b in [IndexBase::One, IndexBase::Zero] {
        d.set_index_base(b).unwrap();
        assert_eq!(d.index_base(), b);
    }
    for f in [FillMode::Upper, FillMode::Lower] {
        d.set_fill_mode(f).unwrap();
        assert_eq!(d.fill_mode(), f);
    }
    for g in [DiagType::Unit, DiagType::NonUnit] {
        d.set_diag_type(g).unwrap();
        assert_eq!(d.diag_type(), g);
    }
    // the setters are independent: changing one leaves the others
    d.set_type(MatType::Triangular).unwrap();
    d.set_index_base(IndexBase::One).unwrap();
    d.set_fill_mode(FillMode::Upper).unwrap();
    d.set_diag_type(DiagType::Unit).unwrap();
    d.set_fill_mode(FillMode::Lower).unwrap();
    assert_eq!(
        (d.ty(), d.index_base(), d.fill_mode(), d.diag_type()),
        (
            MatType::Triangular,
            IndexBase::One,
            FillMode::Lower,
            DiagType::Unit
        )
    );
}

#[test]
fn copy_mat_descr_is_a_deep_copy() {
    let mut src = descr(
        MatType::Triangular,
        FillMode::Upper,
        DiagType::Unit,
        IndexBase::One,
    );
    let dup = s::copy_mat_descr(&src).unwrap();
    assert_eq!(
        (dup.ty(), dup.index_base(), dup.fill_mode(), dup.diag_type()),
        (
            MatType::Triangular,
            IndexBase::One,
            FillMode::Upper,
            DiagType::Unit
        )
    );
    src.set_type(MatType::General).unwrap();
    src.set_index_base(IndexBase::Zero).unwrap();
    src.set_fill_mode(FillMode::Lower).unwrap();
    src.set_diag_type(DiagType::NonUnit).unwrap();
    assert_eq!(
        (dup.ty(), dup.index_base(), dup.fill_mode(), dup.diag_type()),
        (
            MatType::Triangular,
            IndexBase::One,
            FillMode::Upper,
            DiagType::Unit
        )
    );
}

#[test]
fn version_matches_the_installed_header_major() {
    // The installed header (amd-sparse/include/aoclsparse_version.h) says AOCLSPARSE_VERSION_MAJOR 6.
    let ver = s::version();
    assert!(ver.contains("AOCL-Sparse"), "version string: {ver:?}");
    assert!(
        ver.chars().any(|ch| ch.is_ascii_digit()),
        "version string has no number: {ver:?}"
    );
    assert!(ver.contains("6."), "version string: {ver:?}");
}

// ---------------------------------------------------------------- level 1: dot, rotation, strided gather/scatter

/// `f(x, indx, y) = sum_i op(x_i) * y[indx_i]` with `op` = identity (`conj` false) or conjugation (`conj` true).
fn t_dot<T: Lib>(f: fn(&[T], &[I], &[T]) -> Result<T>, conj: bool) {
    let x = vecg::<T>(3, 2);
    let y = vecg::<T>(5, 4);
    let indx: [I; 3] = [4, 0, 2];
    let want = (0..3).fold(ZERO, |acc, i| {
        acc + (if conj { x[i].conj() } else { x[i] }) * y[indx[i] as usize]
    });
    let got = f(&lv::<T>(&x), &indx, &lv::<T>(&y)).unwrap();
    near("dot", 0, got.get(), want, T::TOL);
}

#[test]
fn doti_real_matches_the_serial_sum() {
    t_dot::<f64>(s::doti_f64, false);
    t_dot::<f32>(s::doti_f32, false);
    // x.len() != indx.len() is rejected before the library is called
    assert!(s::doti_f64(&[1.0, 2.0], &[0], &[1.0]).is_err());
    assert!(s::doti_f32(&[1.0], &[0, 1], &[1.0, 2.0]).is_err());
}

#[test]
fn dotui_and_dotci_complex_conjugate_only_the_ci_variant() {
    t_dot::<Complex64>(cxm::dotui_c64, false);
    t_dot::<Complex32>(cxm::dotui_c32, false);
    t_dot::<Complex64>(cxm::dotci_c64, true);
    t_dot::<Complex32>(cxm::dotci_c32, true);
    // the two differ on this data, so a variant that ignored the conjugation would fail one of the two above
    assert!(vecg::<Complex64>(3, 2).iter().any(|z| z.im != 0.0));
    // nnz = 0 is documented invalid_size ("the provided nnz is not positive")
    assert!(cxm::dotui_c64(&[], &[], &[Complex64::new(1.0, 0.0)]).is_err());
    assert!(cxm::dotci_c32(&[], &[], &[Complex32::new(1.0, 0.0)]).is_err());
}

/// Givens rotation of the doc: `x' = c x + s y[indx]`, `y[indx]' = c y[indx] - s x`.
fn t_roti<T: Lib>(f: fn(&mut [T], &[I], &mut [T], T, T) -> Result<()>) {
    let (cc, ss) = (0.6, 0.8);
    let x0 = vecg::<T>(3, 1);
    let y0 = vecg::<T>(5, 3);
    let indx: [I; 3] = [4, 0, 2];
    let (mut x, mut y) = (lv::<T>(&x0), lv::<T>(&y0));
    f(&mut x, &indx, &mut y, T::mk(c(cc, 0.0)), T::mk(c(ss, 0.0))).unwrap();
    let wx: Vec<Cx> = (0..3)
        .map(|i| c(cc, 0.0) * x0[i] + c(ss, 0.0) * y0[indx[i] as usize])
        .collect();
    let mut wy = y0.clone();
    for i in 0..3 {
        wy[indx[i] as usize] = c(cc, 0.0) * y0[indx[i] as usize] - c(ss, 0.0) * x0[i];
    }
    check("roti x", &x, &wx);
    check("roti y", &y, &wy); // entries 1 and 3 are not in indx and must be unchanged
}

#[test]
fn roti_applies_the_givens_rotation() {
    t_roti::<f64>(s::roti_f64);
    t_roti::<f32>(s::roti_f32);
}

fn t_gthrs<T: Lib>(f: fn(&[T], &mut [T], i32) -> Result<()>) {
    let y = vecg::<T>(9, 2);
    let mut x = vec![T::mk(c(-5.0, 0.0)); 3];
    f(&lv::<T>(&y), &mut x, 3).unwrap();
    check("gthrs", &x, &[y[0], y[3], y[6]]);
    let mut x = vec![T::mk(ZERO); 3];
    f(&lv::<T>(&y), &mut x, 1).unwrap();
    check("gthrs stride 1", &x, &y[..3]);
    assert!(f(&lv::<T>(&y), &mut x, -1).is_err()); // a negative stride is invalid_size
}
#[test]
fn gthrs_gathers_with_stride() {
    t_gthrs::<f64>(s::gthrs_f64);
    t_gthrs::<f32>(s::gthrs_f32);
    t_gthrs::<Complex64>(cxm::gthrs_c64);
    t_gthrs::<Complex32>(cxm::gthrs_c32);
}

fn t_sctrs<T: Lib>(f: fn(&[T], &mut [T], i32) -> Result<()>) {
    let x = vecg::<T>(3, 5);
    let mut y = vec![T::mk(c(-7.0, 0.0)); 9];
    f(&lv::<T>(&x), &mut y, 3).unwrap();
    let mut want = vec![c(-7.0, 0.0); 9];
    for i in 0..3 {
        want[3 * i] = x[i];
    }
    check("sctrs", &y, &want); // the entries between the strided slots keep their value
    assert!(f(&lv::<T>(&x), &mut y, 0).is_err()); // a stride that is not positive is invalid_size
}
#[test]
fn sctrs_scatters_with_stride() {
    t_sctrs::<f64>(s::sctrs_f64);
    t_sctrs::<f32>(s::sctrs_f32);
    t_sctrs::<Complex64>(cxm::sctrs_c64);
    t_sctrs::<Complex32>(cxm::sctrs_c32);
}

/// gather-and-zero: `x[i] = y[indx[i]]`, then `y[indx[i]] = 0`.
fn t_gthrz<T: Lib>(f: fn(&mut [T], &[I], &mut [T]) -> Result<()>) {
    let y0 = vecg::<T>(6, 1);
    let indx: [I; 3] = [4, 1, 5];
    let (mut y, mut x) = (lv::<T>(&y0), vec![T::mk(ZERO); 3]);
    f(&mut y, &indx, &mut x).unwrap();
    check("gthrz x", &x, &[y0[4], y0[1], y0[5]]);
    let mut wy = y0.clone();
    for &i in &indx {
        wy[i as usize] = ZERO;
    }
    check("gthrz y", &y, &wy);
}
#[test]
fn gthrz_gathers_then_zeroes_the_source() {
    t_gthrz::<f64>(s::gthrz_f64);
    t_gthrz::<f32>(s::gthrz_f32);
    t_gthrz::<Complex64>(cxm::gthrz_c64);
    t_gthrz::<Complex32>(cxm::gthrz_c32);
}

// ---------------------------------------------------------------- level 2: mv, triangular solves, dotmv

fn t_mv<T: Lib>(f: fn(Trans, T, &T::M, &MatDescr, &[T], T, &mut [T]) -> Result<()>) {
    let a = dm::<T>(&[&[1., 0., 2., 0.], &[0., 3., 0., 4.], &[5., 6., 0., 7.]]);
    for base in [IndexBase::Zero, IndexBase::One] {
        let h = T::build(base, &csr(&a));
        let d = gen_descr(base);
        for op in ops::<T>() {
            let (alpha, beta) = (v::<T>(2.0, 0.5), v::<T>(-1.5, 0.25));
            let (xl, yl) = if op == Trans::No { (4, 3) } else { (3, 4) };
            let (x, y0) = (vecg::<T>(xl, 1), vecg::<T>(yl, 2));
            let mut y = lv::<T>(&y0);
            f(op, T::mk(alpha), &h, &d, &lv::<T>(&x), T::mk(beta), &mut y).unwrap();
            let ax = mv_d(&opd(&a, op), &x);
            let want: Vec<Cx> = (0..yl).map(|i| alpha * ax[i] + beta * y0[i]).collect();
            check(&format!("mv {op:?} {base:?}"), &y, &want);
        }
    }
}
#[test]
fn mv_f64_f32_match_the_dense_product() {
    t_mv::<f64>(s::mv_f64);
    t_mv::<f32>(s::mv_f32);
}

/// Triangular solve checked by multiplying back: `op(A_eff) y == alpha x`, `A_eff` the triangle with a unit diagonal
/// substituted when `diag` is `Unit` (the stored diagonal is then deliberately not 1, to prove it is ignored).
fn t_tri<T: Lib>(
    ty: MatType,
    legacy: bool,
    solve: impl Fn(Trans, T, &Csr, IndexBase, &MatDescr, &[T], &mut [T]) -> Result<()>,
) {
    let lo = dm::<T>(&[
        &[3., 0., 0., 0.],
        &[1., 4., 0., 0.],
        &[0., 2., 5., 0.],
        &[2., 0., 1., 6.],
    ]);
    let up = dm::<T>(&[
        &[3., 1., 0., 2.],
        &[0., 4., 2., 0.],
        &[0., 0., 5., 1.],
        &[0., 0., 0., 6.],
    ]);
    for (fill, a) in [(FillMode::Lower, &lo), (FillMode::Upper, &up)] {
        for diag in [DiagType::NonUnit, DiagType::Unit] {
            for base in [IndexBase::Zero, IndexBase::One]
                .into_iter()
                .filter(|&b| !legacy || b == IndexBase::Zero)
            {
                for op in ops::<T>()
                    .into_iter()
                    .filter(|&o| !legacy || o == Trans::No)
                {
                    let mut eff = a.clone();
                    if diag == DiagType::Unit {
                        (0..4).for_each(|i| eff[i][i] = ONE);
                    }
                    let (alpha, x) = (v::<T>(2.0, 0.5), vecg::<T>(4, 1));
                    let d = descr(ty, fill, diag, base);
                    let mut y = vec![T::mk(ZERO); 4];
                    let tag = format!("tri {fill:?} {diag:?} {base:?} {op:?}");
                    solve(op, T::mk(alpha), &csr(a), base, &d, &lv::<T>(&x), &mut y)
                        .unwrap_or_else(|e| panic!("{tag}: {e:?}"));
                    let back = mv_d(&opd(&eff, op), &cv(&y));
                    let want: Vec<Cx> = x.iter().map(|&xi| alpha * xi).collect();
                    checkc(&tag, &back, &want, T::TOL * 20.0);
                }
            }
        }
    }
}
#[test]
fn csrsv_solves_triangular_systems() {
    fn run<T: Lib + s::Scalar>() {
        // the legacy csrsv solves only op = none, and only with a *general*-typed descriptor (header: "not implemented"
        // otherwise); with a one-based CSR it also answers "not implemented" (FINDING, not asserted), so base zero only
        t_tri::<T>(MatType::General, true, |op, al, cs, base, d, x, y| {
            let (rp, ci) = cs.arrays(base);
            s::csrsv::<T>(op, al, cs.m, &lv::<T>(&cs.v), &ci, &rp, d, x, y)
        });
        let cs = csr(&dm::<T>(&[&[2., 0.], &[1., 3.]]));
        let d = descr(
            MatType::General,
            FillMode::Lower,
            DiagType::NonUnit,
            IndexBase::Zero,
        );
        let mut y = [T::mk(ZERO); 2];
        let x = [T::mk(ONE); 2];
        assert!(s::csrsv::<T>(
            Trans::T,
            T::mk(ONE),
            2,
            &lv::<T>(&cs.v),
            &cs.ci,
            &cs.rp,
            &d,
            &x,
            &mut y
        )
        .is_err());
    }
    run::<f64>();
    run::<f32>();
}
#[test]
fn trsv_handle_solves_triangular_systems() {
    t_tri::<f64>(MatType::Triangular, false, |op, al, cs, base, d, x, y| {
        s::trsv_f64(op, al, &f64::build(base, cs), d, x, y)
    });
    t_tri::<f32>(MatType::Triangular, false, |op, al, cs, base, d, x, y| {
        s::trsv_f32(op, al, &f32::build(base, cs), d, x, y)
    });
    t_tri::<Complex64>(MatType::Triangular, false, |op, al, cs, base, d, x, y| {
        cxm::trsv::<Complex64>(op, al, &Complex64::build(base, cs), d, x, y)
    });
    t_tri::<Complex32>(MatType::Triangular, false, |op, al, cs, base, d, x, y| {
        cxm::trsv::<Complex32>(op, al, &Complex32::build(base, cs), d, x, y)
    });
}
#[test]
fn complex_trsv_rejects_short_buffers() {
    let cs = csr(&dm::<Complex64>(&[&[2., 0.], &[1., 3.]]));
    let h = Complex64::build(IndexBase::Zero, &cs);
    let d = descr(
        MatType::Triangular,
        FillMode::Lower,
        DiagType::NonUnit,
        IndexBase::Zero,
    );
    let b = [Complex64::new(1.0, 0.0); 2];
    let mut x = [Complex64::new(0.0, 0.0); 1];
    assert!(cxm::trsv(Trans::No, Complex64::new(1.0, 0.0), &h, &d, &b, &mut x).is_err());
    assert!(cxm::trsv(
        Trans::No,
        Complex64::new(1.0, 0.0),
        &h,
        &d,
        &b[..1],
        &mut [Complex64::new(0.0, 0.0); 2]
    )
    .is_err());
}

/// Multi-RHS triangular solve with 2 right-hand sides in each storage order: `op(A_eff) X == alpha B`.
fn t_trsm<T: Lib>(
    orders: &[Order],
    solve: impl Fn(Trans, T, &T::M, &MatDescr, Order, &[T], usize, usize, &mut [T], usize) -> Result<()>,
) {
    let lo = dm::<T>(&[
        &[3., 0., 0., 0.],
        &[1., 4., 0., 0.],
        &[0., 2., 5., 0.],
        &[2., 0., 1., 6.],
    ]);
    let up = dm::<T>(&[
        &[3., 1., 0., 2.],
        &[0., 4., 2., 0.],
        &[0., 0., 5., 1.],
        &[0., 0., 0., 6.],
    ]);
    let b: D = (0..4).map(|i| vecg::<T>(2, i)).collect(); // 4x2
    for (fill, a) in [(FillMode::Lower, &lo), (FillMode::Upper, &up)] {
        for diag in [DiagType::NonUnit, DiagType::Unit] {
            for &order in orders {
                for op in ops::<T>() {
                    let mut eff = a.clone();
                    if diag == DiagType::Unit {
                        (0..4).for_each(|i| eff[i][i] = ONE);
                    }
                    let alpha = v::<T>(1.5, -0.5);
                    let ld = if order == Order::RowMajor { 2 } else { 4 };
                    let bb = dense_ld(&b, order, ld);
                    let mut x = vec![T::mk(ZERO); bb.len()];
                    let h = T::build(IndexBase::Zero, &csr(a));
                    let d = descr(MatType::Triangular, fill, diag, IndexBase::Zero);
                    let tag = format!("trsm {fill:?} {diag:?} {order:?} {op:?}");
                    solve(
                        op,
                        T::mk(alpha),
                        &h,
                        &d,
                        order,
                        &lv::<T>(&bb),
                        2,
                        ld,
                        &mut x,
                        ld,
                    )
                    .unwrap_or_else(|e| panic!("{tag}: {e:?}"));
                    let xd = from_ld(&cv(&x), 4, 2, order, ld);
                    let back = mm_d(&opd(&eff, op), &xd);
                    let want: D = b
                        .iter()
                        .map(|r| r.iter().map(|&z| alpha * z).collect())
                        .collect();
                    dense_eq(&tag, &back, &want, T::TOL * 20.0);
                }
            }
        }
    }
}
#[test]
fn trsm_solves_multiple_right_hand_sides() {
    let both = [Order::RowMajor, Order::ColMajor];
    t_trsm::<f64>(&both, |op, al, h, d, o, b, n, ldb, x, ldx| {
        s::trsm_f64(op, al, h, d, o, b, n, ldb, x, ldx)
    });
    t_trsm::<f32>(&both, |op, al, h, d, o, b, n, ldb, x, ldx| {
        s::trsm_f32(op, al, h, d, o, b, n, ldb, x, ldx)
    });
    // the complex wrappers always use row-major storage
    let row = [Order::RowMajor];
    t_trsm::<Complex64>(&row, |op, al, h, d, _o, b, n, ldb, x, ldx| {
        cxm::trsm_c64(op, al, h, d, b, n, ldb, x, ldx)
    });
    t_trsm::<Complex32>(&row, |op, al, h, d, _o, b, n, ldb, x, ldx| {
        cxm::trsm_c32(op, al, h, d, b, n, ldb, x, ldx)
    });
}

/// `y := alpha op(A) x + beta y`, then `d = sum_{i<min(m,n)} conj(x_i) y_i` (conj only for complex), on a 3x4 matrix.
fn t_dotmv<T: Lib>(f: fn(Trans, T, &T::M, &MatDescr, &[T], T, &mut [T], &mut T) -> Result<()>) {
    let a = dm::<T>(&[&[1., 0., 2., 0.], &[0., 3., 0., 4.], &[5., 6., 0., 7.]]);
    let h = T::build(IndexBase::Zero, &csr(&a));
    let dsc = gen_descr(IndexBase::Zero);
    let ops = if T::CPLX {
        [Trans::No, Trans::C]
    } else {
        [Trans::No, Trans::T]
    };
    for op in ops {
        let (alpha, beta) = (v::<T>(2.0, 0.5), v::<T>(-1.5, 0.25));
        let (xl, yl) = if op == Trans::No { (4, 3) } else { (3, 4) };
        let (x, y0) = (vecg::<T>(xl, 1), vecg::<T>(yl, 2));
        let (mut y, mut dd) = (lv::<T>(&y0), T::mk(c(99.0, 99.0)));
        f(
            op,
            T::mk(alpha),
            &h,
            &dsc,
            &lv::<T>(&x),
            T::mk(beta),
            &mut y,
            &mut dd,
        )
        .unwrap();
        let ax = mv_d(&opd(&a, op), &x);
        let want: Vec<Cx> = (0..yl).map(|i| alpha * ax[i] + beta * y0[i]).collect();
        check(&format!("dotmv y {op:?}"), &y, &want);
        let wd = (0..xl.min(yl)).fold(ZERO, |acc, i| acc + x[i].conj() * want[i]);
        near(&format!("dotmv d {op:?}"), 0, dd.get(), wd, T::TOL * 10.0);
    }
    // the descriptor's index base must match the matrix's (documented invalid_value)
    let (mut y, mut dd) = (vec![T::mk(ZERO); 3], T::mk(ZERO));
    assert!(f(
        Trans::No,
        T::mk(ONE),
        &h,
        &gen_descr(IndexBase::One),
        &lv::<T>(&vecg::<T>(4, 1)),
        T::mk(ZERO),
        &mut y,
        &mut dd
    )
    .is_err());
}
#[test]
fn dotmv_returns_the_updated_product_and_its_dot() {
    t_dotmv::<f64>(s::dotmv_f64);
    t_dotmv::<f32>(s::dotmv_f32);
    t_dotmv::<Complex64>(cxm::dotmv_c64);
    t_dotmv::<Complex32>(cxm::dotmv_c32);
}
