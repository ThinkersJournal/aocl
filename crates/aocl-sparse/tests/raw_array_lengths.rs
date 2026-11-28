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

/// Which argument to break: `Ok` = nothing (control); `Short(arg)` = that array one element short, at the guard page; `Idx(arg, pos, v)`
/// = `index_array[pos] = v` (an out-of-range index or pointer).
#[derive(Clone, Copy, Debug)]
enum Brk {
    Ok,
    Short(&'static str),
    Idx(&'static str, usize, I),
}

macro_rules! family {
    ($m:ident, $T:ty, $one:expr, csr2ell = $csr2ell:expr, csr2dia = $csr2dia:expr) => {
        mod $m {
            use super::*;
            fn v(x: f64) -> $T {
                ($one)(x)
            }
            fn vals(n: usize) -> Vec<$T> {
                (0..n).map(|i| v(1.0 + i as f64)).collect()
            }

            // 3 x 4: rp/ci as in the wrapper tests; one-based variants are not exercised (the wrappers' own tests say the legacy
            // functions answer "not implemented" for them).
            fn csrmv_with(b: Brk) -> Result<()> {
                let (mut rp, mut ci): (Vec<I>, Vec<I>) = (vec![0, 2, 3, 5], vec![0, 2, 1, 0, 3]);
                let (val, x, mut y) = (vals(5), vals(4), vec![v(0.0); 3]);
                if let Brk::Idx(a, p, k) = b {
                    match a {
                        "ci" => ci[p] = k,
                        "rp" => rp[p] = k,
                        _ => unreachable!(),
                    }
                }
                let d = descr(MatType::General);
                let call = |val: &[$T], ci: &[I], rp: &[I], x: &[$T], y: &mut [$T]| {
                    csrmv::<$T>(v(1.0), 3, 4, val, ci, rp, &d, x, v(0.0), y)
                };
                match b {
                    Brk::Short("val") => at_guard(&val, 4, |s| call(s, &ci, &rp, &x, &mut y)),
                    Brk::Short("ci") => at_guard(&ci, 4, |s| call(&val, s, &rp, &x, &mut y)),
                    Brk::Short("rp") => at_guard(&rp, 3, |s| call(&val, &ci, s, &x, &mut y)),
                    Brk::Short("x") => at_guard(&x, 3, |s| call(&val, &ci, &rp, s, &mut y)),
                    Brk::Short("y") => {
                        let yy = vec![v(0.0); 3];
                        at_guard(&yy, 2, |s| call(&val, &ci, &rp, &x, s))
                    }
                    Brk::Idx("ci", ..) | Brk::Idx("rp", ..) => {
                        at_guard(&x, 4, |s| call(&val, &ci, &rp, s, &mut y))
                    }
                    _ => call(&val, &ci, &rp, &x, &mut y),
                }
            }
            #[test]
            fn csrmv_control() {
                csrmv_with(Brk::Ok).unwrap();
            }
            macro_rules! t {
                ($name:ident, $f:ident, $brk:expr) => {
                    #[test]
                    fn $name() {
                        refused(stringify!($name), $f($brk));
                    }
                };
            }
            t!(csrmv_short_val, csrmv_with, Brk::Short("val"));
            t!(csrmv_short_ci, csrmv_with, Brk::Short("ci"));
            t!(csrmv_short_rp, csrmv_with, Brk::Short("rp"));
            t!(csrmv_short_x, csrmv_with, Brk::Short("x"));
            t!(csrmv_short_y, csrmv_with, Brk::Short("y"));
            t!(csrmv_col_past_n, csrmv_with, Brk::Idx("ci", 1, 4));
            t!(csrmv_col_negative, csrmv_with, Brk::Idx("ci", 1, -2));
            t!(csrmv_rowptr_past_nnz, csrmv_with, Brk::Idx("rp", 3, 99));
            t!(csrmv_rowptr_decreasing, csrmv_with, Brk::Idx("rp", 2, 1));

            // csrsv: 3 x 3 lower triangular, general descriptor, op none
            fn csrsv_with(b: Brk) -> Result<()> {
                let (mut rp, mut ci): (Vec<I>, Vec<I>) = (vec![0, 1, 3, 6], vec![0, 0, 1, 0, 1, 2]);
                let (val, x, mut y) = (vals(6), vals(3), vec![v(0.0); 3]);
                if let Brk::Idx(a, p, k) = b {
                    match a {
                        "ci" => ci[p] = k,
                        "rp" => rp[p] = k,
                        _ => unreachable!(),
                    }
                }
                let d = descr(MatType::General);
                let call = |val: &[$T], ci: &[I], rp: &[I], x: &[$T], y: &mut [$T]| {
                    csrsv::<$T>(Trans::No, v(1.0), 3, val, ci, rp, &d, x, y)
                };
                match b {
                    Brk::Short("val") => at_guard(&val, 5, |s| call(s, &ci, &rp, &x, &mut y)),
                    Brk::Short("ci") => at_guard(&ci, 5, |s| call(&val, s, &rp, &x, &mut y)),
                    Brk::Short("rp") => at_guard(&rp, 3, |s| call(&val, &ci, s, &x, &mut y)),
                    Brk::Short("x") => at_guard(&x, 2, |s| call(&val, &ci, &rp, s, &mut y)),
                    Brk::Short("y") => {
                        let yy = vec![v(0.0); 3];
                        at_guard(&yy, 2, |s| call(&val, &ci, &rp, &x, s))
                    }
                    Brk::Idx(..) => at_guard(&x, 3, |s| call(&val, &ci, &rp, s, &mut y)),
                    _ => call(&val, &ci, &rp, &x, &mut y),
                }
            }
            #[test]
            fn csrsv_control() {
                csrsv_with(Brk::Ok).unwrap();
            }
            t!(csrsv_short_val, csrsv_with, Brk::Short("val"));
            t!(csrsv_short_ci, csrsv_with, Brk::Short("ci"));
            t!(csrsv_short_rp, csrsv_with, Brk::Short("rp"));
            t!(csrsv_short_x, csrsv_with, Brk::Short("x"));
            t!(csrsv_short_y, csrsv_with, Brk::Short("y"));
            t!(csrsv_col_past_m, csrsv_with, Brk::Idx("ci", 3, 3));
            t!(csrsv_rowptr_past_nnz, csrsv_with, Brk::Idx("rp", 3, 99));

            // ellmv from a real csr2ell result: 3 x 4, width 3
            fn ell_arrays() -> (Vec<I>, Vec<$T>) {
                let (rp, ci): (Vec<I>, Vec<I>) = (vec![0, 2, 3, 5], vec![0, 2, 1, 0, 3]);
                let (mut col, mut val) = (vec![0 as I; 9], vec![v(0.0); 9]);
                ($csr2ell)(
                    3,
                    &descr(MatType::General),
                    &rp,
                    &ci,
                    &vals(5),
                    &mut col,
                    &mut val,
                    3,
                )
                .unwrap();
                (col, val)
            }
            fn ellmv_with(b: Brk) -> Result<()> {
                let (mut col, val) = ell_arrays();
                let (x, mut y) = (vals(4), vec![v(0.0); 3]);
                if let Brk::Idx(_, p, k) = b {
                    col[p] = k;
                }
                let d = descr(MatType::General);
                let call = |val: &[$T], col: &[I], x: &[$T], y: &mut [$T]| {
                    ellmv::<$T>(Trans::No, v(1.0), 3, 4, val, col, 3, &d, x, v(0.0), y)
                };
                match b {
                    Brk::Short("val") => at_guard(&val, 8, |s| call(s, &col, &x, &mut y)),
                    Brk::Short("col") => at_guard(&col, 8, |s| call(&val, s, &x, &mut y)),
                    Brk::Short("x") => at_guard(&x, 3, |s| call(&val, &col, s, &mut y)),
                    Brk::Short("y") => {
                        let yy = vec![v(0.0); 3];
                        at_guard(&yy, 2, |s| call(&val, &col, &x, s))
                    }
                    Brk::Idx(..) => at_guard(&x, 4, |s| call(&val, &col, s, &mut y)),
                    _ => call(&val, &col, &x, &mut y),
                }
            }
            #[test]
            fn ellmv_control() {
                ellmv_with(Brk::Ok).unwrap();
            }
            t!(ellmv_short_val, ellmv_with, Brk::Short("val"));
            t!(ellmv_short_col, ellmv_with, Brk::Short("col"));
            t!(ellmv_short_x, ellmv_with, Brk::Short("x"));
            t!(ellmv_short_y, ellmv_with, Brk::Short("y"));
            t!(ellmv_col_past_n, ellmv_with, Brk::Idx("col", 0, 4));

            // csr2ell: outputs one element short
            fn csr2ell_with(b: Brk) -> Result<()> {
                let (rp, ci): (Vec<I>, Vec<I>) = (vec![0, 2, 3, 5], vec![0, 2, 1, 0, 3]);
                let (val, col0, val0) = (vals(5), vec![0 as I; 9], vec![v(0.0); 9]);
                let d = descr(MatType::General);
                let call = |rp: &[I], ci: &[I], val: &[$T], oc: &mut [I], ov: &mut [$T]| {
                    ($csr2ell)(3, &d, rp, ci, val, oc, ov, 3)
                };
                match b {
                    Brk::Short("oc") => {
                        let mut ov = val0.clone();
                        at_guard(&col0, 8, |s| call(&rp, &ci, &val, s, &mut ov))
                    }
                    Brk::Short("ov") => {
                        let mut oc = col0.clone();
                        at_guard(&val0, 8, |s| call(&rp, &ci, &val, &mut oc, s))
                    }
                    Brk::Short("rp") => {
                        let (mut oc, mut ov) = (col0.clone(), val0.clone());
                        at_guard(&rp, 3, |s| call(s, &ci, &val, &mut oc, &mut ov))
                    }
                    Brk::Short("ci") => {
                        let (mut oc, mut ov) = (col0.clone(), val0.clone());
                        at_guard(&ci, 4, |s| call(&rp, s, &val, &mut oc, &mut ov))
                    }
                    Brk::Short("val") => {
                        let (mut oc, mut ov) = (col0.clone(), val0.clone());
                        at_guard(&val, 4, |s| call(&rp, &ci, s, &mut oc, &mut ov))
                    }
                    _ => {
                        let (mut oc, mut ov) = (col0.clone(), val0.clone());
                        call(&rp, &ci, &val, &mut oc, &mut ov)
                    }
                }
            }
            #[test]
            fn csr2ell_control() {
                csr2ell_with(Brk::Ok).unwrap();
            }
            t!(csr2ell_short_out_col, csr2ell_with, Brk::Short("oc"));
            t!(csr2ell_short_out_val, csr2ell_with, Brk::Short("ov"));
            t!(csr2ell_short_rp, csr2ell_with, Brk::Short("rp"));
            t!(csr2ell_short_ci, csr2ell_with, Brk::Short("ci"));
            t!(csr2ell_short_val, csr2ell_with, Brk::Short("val"));

            // csr2dia: 4 x 4 with diagonals -1, 0, +2 (3 diagonals), outputs: offsets (3) and values (12)
            fn csr2dia_with(b: Brk) -> Result<()> {
                let rp: Vec<I> = vec![0, 2, 5, 7, 9];
                let ci: Vec<I> = vec![0, 2, 0, 1, 3, 1, 2, 2, 3];
                let (val, off0, dv0) = (vals(9), vec![0 as I; 3], vec![v(-9.0); 12]);
                let d = descr(MatType::General);
                let call = |rp: &[I], ci: &[I], val: &[$T], off: &mut [I], dv: &mut [$T]| {
                    ($csr2dia)(4, 4, &d, rp, ci, val, 3, off, dv)
                };
                match b {
                    Brk::Short("off") => {
                        let mut dv = dv0.clone();
                        at_guard(&off0, 2, |s| call(&rp, &ci, &val, s, &mut dv))
                    }
                    Brk::Short("dv") => {
                        let mut off = off0.clone();
                        at_guard(&dv0, 11, |s| call(&rp, &ci, &val, &mut off, s))
                    }
                    Brk::Short("rp") => {
                        let (mut off, mut dv) = (off0.clone(), dv0.clone());
                        at_guard(&rp, 4, |s| call(s, &ci, &val, &mut off, &mut dv))
                    }
                    Brk::Short("ci") => {
                        let (mut off, mut dv) = (off0.clone(), dv0.clone());
                        at_guard(&ci, 8, |s| call(&rp, s, &val, &mut off, &mut dv))
                    }
                    Brk::Short("val") => {
                        let (mut off, mut dv) = (off0.clone(), dv0.clone());
                        at_guard(&val, 8, |s| call(&rp, &ci, s, &mut off, &mut dv))
                    }
                    _ => {
                        let (mut off, mut dv) = (off0.clone(), dv0.clone());
                        call(&rp, &ci, &val, &mut off, &mut dv)
                    }
                }
            }
            #[test]
            fn csr2dia_control() {
                csr2dia_with(Brk::Ok).unwrap();
            }
            t!(csr2dia_short_offsets, csr2dia_with, Brk::Short("off"));
            t!(csr2dia_short_values, csr2dia_with, Brk::Short("dv"));
            t!(csr2dia_short_rp, csr2dia_with, Brk::Short("rp"));
            t!(csr2dia_short_ci, csr2dia_with, Brk::Short("ci"));
            t!(csr2dia_short_val, csr2dia_with, Brk::Short("val"));
        }
    };
}

family!(
    real_f64,
    f64,
    |x: f64| x,
    csr2ell = csr2ell_f64,
    csr2dia = csr2dia_f64
);
family!(
    real_f32,
    f32,
    |x: f64| x as f32,
    csr2ell = csr2ell_f32,
    csr2dia = csr2dia_f32
);
