//! RED-FIRST proof for the length rule of the matrix-handle vector functions (`mv`, `trsv`, `symgs`, `symgs_mv`, `dotmv`; every
//! precision): the library does not check that `x`, `y`, `b` are as long as the matrix needs, so a SAFE call with a short slice read
//! or wrote past the slice. Each case puts the short slice at the END of a committed page followed by a PAGE_NOACCESS page, so an
//! access past it is an access violation (0xc0000005) that kills the process; after the fix each case returns
//! `Err(InvalidArgument)` before the library is called. The matrix is 4 x 4 diagonal; the short slice has 3 elements; every other slice has 4. Each case is its own test (run
//! with `-- --exact <name>` to see a crash name itself). Windows only (VirtualAlloc); Linux unmeasured. No `unsafe` outside the page helper.
#![cfg(windows)]
use aocl_sparse::complex::{self as cx, ComplexSparseMatrix};
use aocl_sparse::*;
use core::ffi::c_void;

extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
    fn VirtualFree(addr: *mut c_void, size: usize, ty: u32) -> i32;
}
const PAGE: usize = 4096;

/// Run `f` with a slice of `n` elements that ENDS at a committed page, the next page being PAGE_NOACCESS.
fn at_guard<T: Copy, R>(n: usize, fill: T, f: impl FnOnce(&mut [T]) -> R) -> R {
    assert!(n * core::mem::size_of::<T>() <= PAGE);
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
        for i in 0..n {
            start.add(i).write(fill);
        }
        let r = f(core::slice::from_raw_parts_mut(start, n));
        VirtualFree(base, 0, 0x8000);
        r
    }
}

fn descr(ty: MatType) -> MatDescr {
    let mut d = MatDescr::new().unwrap();
    d.set_type(ty).unwrap();
    d.set_fill_mode(FillMode::Lower).unwrap();
    d.set_diag_type(DiagType::NonUnit).unwrap();
    d
}

fn refused<T: core::fmt::Debug>(what: &str, r: Result<T>) {
    assert!(matches!(r, Err(Error::InvalidArgument(_))), "{what}: {r:?}");
}

const RP: [i32; 5] = [0, 1, 2, 3, 4];
const CI: [i32; 4] = [0, 1, 2, 3];

macro_rules! family {
    ($m:ident, $T:ty, $Mat:ty, $symty:expr, $val:expr,
     mv = $mv:expr, trsv = $trsv:expr, symgs = $symgs:expr, symgs_mv = $symgs_mv:expr, dotmv = $dotmv:expr) => {
        mod $m {
            use super::*;
            fn mat() -> $Mat {
                let v: Vec<$T> = (0..4).map(|_| ($val)(2.0)).collect();
                <$Mat>::from_csr(IndexBase::Zero, 4, 4, &RP, &CI, &v).unwrap()
            }
            fn full() -> Vec<$T> {
                vec![($val)(1.0); 4]
            }
            fn one() -> $T {
                ($val)(1.0)
            }
            fn zero() -> $T {
                ($val)(0.0)
            }

            #[test]
            fn mv_short_x() {
                let (a, d) = (mat(), descr(MatType::General));
                let mut y = full();
                refused(
                    "mv x",
                    at_guard::<$T, _>(3, one(), |x| {
                        ($mv)(Trans::No, one(), &a, &d, x, zero(), &mut y)
                    }),
                );
            }
            #[test]
            fn mv_short_y() {
                let (a, d) = (mat(), descr(MatType::General));
                let x = full();
                refused(
                    "mv y",
                    at_guard::<$T, _>(3, one(), |y| ($mv)(Trans::No, one(), &a, &d, &x, zero(), y)),
                );
            }
            #[test]
            fn trsv_short_b() {
                let (a, d) = (mat(), descr(MatType::Triangular));
                let mut x = full();
                refused(
                    "trsv b",
                    at_guard::<$T, _>(3, one(), |b| ($trsv)(Trans::No, one(), &a, &d, b, &mut x)),
                );
            }
            #[test]
            fn trsv_short_x() {
                let (a, d) = (mat(), descr(MatType::Triangular));
                let b = full();
                refused(
                    "trsv x",
                    at_guard::<$T, _>(3, one(), |x| ($trsv)(Trans::No, one(), &a, &d, &b, x)),
                );
            }
            #[test]
            fn symgs_short_b() {
                let (a, d) = (mat(), descr($symty));
                let mut x = full();
                refused(
                    "symgs b",
                    at_guard::<$T, _>(3, one(), |b| ($symgs)(Trans::No, &a, &d, one(), b, &mut x)),
                );
            }
            #[test]
            fn symgs_short_x() {
                let (a, d) = (mat(), descr($symty));
                let b = full();
                refused(
                    "symgs x",
                    at_guard::<$T, _>(3, one(), |x| ($symgs)(Trans::No, &a, &d, one(), &b, x)),
                );
            }
            #[test]
            fn symgs_mv_short_b() {
                let (a, d) = (mat(), descr($symty));
                let (mut x, mut y) = (full(), full());
                refused(
                    "symgs_mv b",
                    at_guard::<$T, _>(3, one(), |b| {
                        ($symgs_mv)(Trans::No, &a, &d, one(), b, &mut x, &mut y)
                    }),
                );
            }
            #[test]
            fn symgs_mv_short_x() {
                let (a, d) = (mat(), descr($symty));
                let (b, mut y) = (full(), full());
                refused(
                    "symgs_mv x",
                    at_guard::<$T, _>(3, one(), |x| {
                        ($symgs_mv)(Trans::No, &a, &d, one(), &b, x, &mut y)
                    }),
                );
            }
            #[test]
            fn symgs_mv_short_y() {
                let (a, d) = (mat(), descr($symty));
                let (b, mut x) = (full(), full());
                refused(
                    "symgs_mv y",
                    at_guard::<$T, _>(3, one(), |y| {
                        ($symgs_mv)(Trans::No, &a, &d, one(), &b, &mut x, y)
                    }),
                );
            }
            #[test]
            fn dotmv_short_x() {
                let (a, d) = (mat(), descr(MatType::General));
                let (mut y, mut dd) = (full(), zero());
                refused(
                    "dotmv x",
                    at_guard::<$T, _>(3, one(), |x| {
                        ($dotmv)(Trans::No, one(), &a, &d, x, zero(), &mut y, &mut dd)
                    }),
                );
            }
            #[test]
            fn dotmv_short_y() {
                let (a, d) = (mat(), descr(MatType::General));
                let (x, mut dd) = (full(), zero());
                refused(
                    "dotmv y",
                    at_guard::<$T, _>(3, one(), |y| {
                        ($dotmv)(Trans::No, one(), &a, &d, &x, zero(), y, &mut dd)
                    }),
                );
            }
            /// Control: exact lengths still work (a check that rejects everything would pass the cases above).
            #[test]
            fn right_lengths_still_succeed() {
                let (a, g, s) = (mat(), descr(MatType::General), descr($symty));
                let t = descr(MatType::Triangular);
                let (x0, b) = (full(), full());
                let (mut y, mut x, mut y2, mut dd) = (full(), full(), full(), zero());
                ($mv)(Trans::No, one(), &a, &g, &x0, zero(), &mut y).unwrap();
                ($trsv)(Trans::No, one(), &a, &t, &b, &mut x).unwrap();
                ($symgs)(Trans::No, &a, &s, one(), &b, &mut x).unwrap();
                ($symgs_mv)(Trans::No, &a, &s, one(), &b, &mut x, &mut y2).unwrap();
                ($dotmv)(Trans::No, one(), &a, &g, &x0, zero(), &mut y, &mut dd).unwrap();
            }
        }
    };
}

family!(
    real_f64,
    f64,
    SparseMatrix<f64>,
    MatType::Symmetric,
    |v: f64| v,
    mv = mv_f64,
    trsv = trsv_f64,
    symgs = symgs_f64,
    symgs_mv = symgs_mv_f64,
    dotmv = dotmv_f64
);
family!(
    real_f32,
    f32,
    SparseMatrix<f32>,
    MatType::Symmetric,
    |v: f64| v as f32,
    mv = mv_f32,
    trsv = trsv_f32,
    symgs = symgs_f32,
    symgs_mv = symgs_mv_f32,
    dotmv = dotmv_f32
);
family!(
    cplx_c64,
    Complex64,
    ComplexSparseMatrix<Complex64>,
    MatType::Hermitian,
    |v: f64| Complex64::new(v, 0.0),
    mv = cx::mv,
    trsv = cx::trsv,
    symgs = cx::symgs_c64,
    symgs_mv = cx::symgs_mv_c64,
    dotmv = cx::dotmv_c64
);
family!(
    cplx_c32,
    Complex32,
    ComplexSparseMatrix<Complex32>,
    MatType::Hermitian,
    |v: f64| Complex32::new(v as f32, 0.0),
    mv = cx::mv,
    trsv = cx::trsv,
    symgs = cx::symgs_c32,
    symgs_mv = cx::symgs_mv_c32,
    dotmv = cx::dotmv_c32
);
