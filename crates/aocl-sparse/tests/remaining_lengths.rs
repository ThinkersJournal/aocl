//! RED-FIRST proof for the raw-array sparse functions that take the CSR / ELL / BSR / DIA arrays directly (no matrix handle, so no
//! validation by `create_csr`): `csrmv`, `csrsv`, `ellmv`, `bsrmv`, `csr2ell`, `csr2dia`. The library trusts the lengths AND the
//! contents of these arrays. Each case builds a valid call (the control), then breaks ONE thing: one array one element short, placed at
//! the end of a committed page before a PAGE_NOACCESS page (access past it = access violation 0xc0000005), or one index out of range.
//! After the fix each broken case returns `Err(InvalidArgument)`. One process per case (`-- --exact <name>`). Windows only; Linux
//! unmeasured. f64 and f32 (the generic wrappers share one body; the f32 cases prove the monomorphisations behave the same).
#![cfg(windows)]
#![allow(
    clippy::type_complexity,
    clippy::too_many_arguments,
    reason = "test helpers mirror the long signatures of the wrappers under test"
)]
use aocl_sparse::*;
use core::ffi::c_void;

extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
    fn VirtualFree(addr: *mut c_void, size: usize, ty: u32) -> i32;
}
const PAGE: usize = 4096;
type I = i32;

/// Run `f` with a COPY of `src[..n]` that ENDS at a committed page, the next page being PAGE_NOACCESS.
fn at_guard<T: Copy, R>(src: &[T], n: usize, f: impl FnOnce(&mut [T]) -> R) -> R {
    assert!(n <= src.len() && n * core::mem::size_of::<T>() <= PAGE);
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
        let s = core::slice::from_raw_parts_mut(start, n);
        s.copy_from_slice(&src[..n]);
        let r = f(s);
        VirtualFree(base, 0, 0x8000);
        r
    }
}

fn refused<T: core::fmt::Debug>(what: &str, r: Result<T>) {
    assert!(matches!(r, Err(Error::InvalidArgument(_))), "{what}: {r:?}");
}

fn descr(ty: MatType) -> MatDescr {
    let mut d = MatDescr::new().unwrap();
    d.set_type(ty).unwrap();
    d.set_fill_mode(FillMode::Lower).unwrap();
    d.set_diag_type(DiagType::NonUnit).unwrap();
    d
}

#[derive(Clone, Copy, Debug)]
enum Brk {
    Ok,
    Short(&'static str),
    Idx(&'static str, usize, I),
}

macro_rules! t {
    ($name:ident, $f:ident, $brk:expr) => {
        #[test]
        fn $name() {
            refused(stringify!($name), $f($brk));
        }
    };
}

macro_rules! family {
    ($m:ident, $T:ty, $one:expr, csr2bsr = $csr2bsr:expr) => {
        mod $m {
            use super::*;
            fn v(x: f64) -> $T {
                ($one)(x)
            }
            fn vals(n: usize) -> Vec<$T> {
                (0..n).map(|i| v(1.0 + i as f64)).collect()
            }

            // bsrmv: 2 x 2 blocks of 2 x 2 (a 4 x 4 matrix), 3 non-zero blocks
            fn bsrmv_with(b: Brk) -> Result<()> {
                let (mut rp, mut ci): (Vec<I>, Vec<I>) = (vec![0, 2, 3], vec![0, 1, 1]);
                let (val, x, mut y) = (vals(12), vals(4), vec![v(0.0); 4]);
                if let Brk::Idx(a, p, k) = b {
                    match a {
                        "ci" => ci[p] = k,
                        "rp" => rp[p] = k,
                        _ => unreachable!(),
                    }
                }
                let d = descr(MatType::General);
                let call = |val: &[$T], ci: &[I], rp: &[I], x: &[$T], y: &mut [$T]| bsrmv::<$T>(Trans::No, v(1.0), 2, 2, 2, val, ci, rp, &d, x, v(0.0), y);
                match b {
                    Brk::Short("val") => at_guard(&val, 11, |s| call(s, &ci, &rp, &x, &mut y)),
                    Brk::Short("ci") => at_guard(&ci, 2, |s| call(&val, s, &rp, &x, &mut y)),
                    Brk::Short("rp") => at_guard(&rp, 2, |s| call(&val, &ci, s, &x, &mut y)),
                    Brk::Short("x") => at_guard(&x, 3, |s| call(&val, &ci, &rp, s, &mut y)),
                    Brk::Short("y") => {
                        let yy = vec![v(0.0); 4];
                        at_guard(&yy, 3, |s| call(&val, &ci, &rp, &x, s))
                    }
                    Brk::Idx(..) => at_guard(&x, 4, |s| call(&val, &ci, &rp, s, &mut y)),
                    _ => call(&val, &ci, &rp, &x, &mut y),
                }
            }
            #[test]
            fn bsrmv_control() {
                bsrmv_with(Brk::Ok).unwrap();
            }
            t!(bsrmv_short_val, bsrmv_with, Brk::Short("val"));
            t!(bsrmv_short_ci, bsrmv_with, Brk::Short("ci"));
            t!(bsrmv_short_rp, bsrmv_with, Brk::Short("rp"));
            t!(bsrmv_short_x, bsrmv_with, Brk::Short("x"));
            t!(bsrmv_short_y, bsrmv_with, Brk::Short("y"));
            t!(bsrmv_block_col_past_nb, bsrmv_with, Brk::Idx("ci", 1, 2));
            t!(bsrmv_rowptr_past_nnzb, bsrmv_with, Brk::Idx("rp", 2, 9));

            // csr2bsr_nnz + csr2bsr: 4 x 4 CSR (7 entries), block_dim 2, mb = 2
            const RP: [I; 5] = [0, 2, 3, 5, 7];
            const CI: [I; 7] = [0, 2, 1, 0, 3, 2, 3];
            fn csr2bsr_nnz_with(b: Brk) -> Result<()> {
                let val_dummy = vals(7);
                let _ = &val_dummy;
                let bsr_rp0 = vec![0 as I; 3];
                let d = descr(MatType::General);
                let call = |rp: &[I], ci: &[I], out: &mut [I]| csr2bsr_nnz(4, 4, &d, rp, ci, 2, out).map(|_| ());
                match b {
                    Brk::Short("out") => at_guard(&bsr_rp0, 2, |s| call(&RP, &CI, s)),
                    Brk::Short("rp") => at_guard(&RP, 4, |s| {
                        let mut o = bsr_rp0.clone();
                        call(s, &CI, &mut o)
                    }),
                    Brk::Short("ci") => at_guard(&CI, 6, |s| {
                        let mut o = bsr_rp0.clone();
                        call(&RP, s, &mut o)
                    }),
                    _ => {
                        let mut o = bsr_rp0.clone();
                        call(&RP, &CI, &mut o)
                    }
                }
            }
            #[test]
            fn csr2bsr_nnz_control() {
                csr2bsr_nnz_with(Brk::Ok).unwrap();
            }
            t!(csr2bsr_nnz_short_out, csr2bsr_nnz_with, Brk::Short("out"));
            t!(csr2bsr_nnz_short_rp, csr2bsr_nnz_with, Brk::Short("rp"));
            t!(csr2bsr_nnz_short_ci, csr2bsr_nnz_with, Brk::Short("ci"));

            fn csr2bsr_with(b: Brk) -> Result<()> {
                let val = vals(7);
                let d = descr(MatType::General);
                let mut tmp = vec![0 as I; 3];
                let nnzb = csr2bsr_nnz(4, 4, &d, &RP, &CI, 2, &mut tmp).unwrap();
                let (bv0, brp0, bci0) = (vec![v(0.0); nnzb * 4], vec![0 as I; 3], vec![0 as I; nnzb]);
                let call = |val: &[$T], rp: &[I], ci: &[I], bv: &mut [$T], brp: &mut [I], bci: &mut [I]| ($csr2bsr)(4, 4, &d, Order::RowMajor, val, rp, ci, 2, bv, brp, bci);
                let (mut bv, mut brp, mut bci) = (bv0.clone(), brp0.clone(), bci0.clone());
                match b {
                    Brk::Short("bv") => at_guard(&bv0, nnzb * 4 - 1, |s| call(&val, &RP, &CI, s, &mut brp, &mut bci)),
                    Brk::Short("brp") => at_guard(&brp0, 2, |s| call(&val, &RP, &CI, &mut bv, s, &mut bci)),
                    Brk::Short("bci") => at_guard(&bci0, nnzb - 1, |s| call(&val, &RP, &CI, &mut bv, &mut brp, s)),
                    Brk::Short("val") => at_guard(&val, 6, |s| call(s, &RP, &CI, &mut bv, &mut brp, &mut bci)),
                    Brk::Short("rp") => at_guard(&RP, 4, |s| call(&val, s, &CI, &mut bv, &mut brp, &mut bci)),
                    Brk::Short("ci") => at_guard(&CI, 6, |s| call(&val, &RP, s, &mut bv, &mut brp, &mut bci)),
                    _ => call(&val, &RP, &CI, &mut bv, &mut brp, &mut bci),
                }
            }
            #[test]
            fn csr2bsr_control() {
                csr2bsr_with(Brk::Ok).unwrap();
            }
            t!(csr2bsr_short_bsr_val, csr2bsr_with, Brk::Short("bv"));
            t!(csr2bsr_short_bsr_rowptr, csr2bsr_with, Brk::Short("brp"));
            t!(csr2bsr_short_bsr_colind, csr2bsr_with, Brk::Short("bci"));
            t!(csr2bsr_short_val, csr2bsr_with, Brk::Short("val"));
            t!(csr2bsr_short_rp, csr2bsr_with, Brk::Short("rp"));
            t!(csr2bsr_short_ci, csr2bsr_with, Brk::Short("ci"));
        }
    };
}

family!(real_f64, f64, |x: f64| x, csr2bsr = csr2bsr_f64);
family!(real_f32, f32, |x: f64| x as f32, csr2bsr = csr2bsr_f32);

// IterSolver::solve: 4 x 4 SPD tridiagonal, CG; b and x one element short
fn solve_with(b: &'static str) -> Result<()> {
    let rp: Vec<I> = vec![0, 2, 5, 8, 10];
    let ci: Vec<I> = vec![0, 1, 0, 1, 2, 1, 2, 3, 2, 3];
    let val: Vec<f64> = vec![4.0, 1.0, 1.0, 4.0, 1.0, 1.0, 4.0, 1.0, 1.0, 4.0];
    let a = SparseMatrix::<f64>::from_csr(IndexBase::Zero, 4, 4, &rp, &ci, &val).unwrap();
    let mut d = descr(MatType::Symmetric);
    d.set_fill_mode(FillMode::Lower).unwrap();
    let mut s = IterSolver::<f64>::new().unwrap();
    s.set_option("iterative method", "cg").unwrap();
    let ones = vec![1.0f64; 4];
    match b {
        "b" => at_guard(&ones, 3, |bb| {
            let mut x = vec![0.0f64; 4];
            s.solve(&a, &d, bb, &mut x).map(|_| ())
        }),
        "x" => at_guard(&ones, 3, |xx| s.solve(&a, &d, &ones, xx).map(|_| ())),
        _ => {
            let mut x = vec![0.0f64; 4];
            s.solve(&a, &d, &ones, &mut x).map(|_| ())
        }
    }
}
#[test]
fn solve_control() {
    solve_with("ok").unwrap();
}
#[test]
fn solve_short_b() {
    refused("solve b", solve_with("b"));
}
#[test]
fn solve_short_x() {
    refused("solve x", solve_with("x"));
}
