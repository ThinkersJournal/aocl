//! Guard-page harness for the aocl-fft safe functions and plan executors (f64 and f32), written by hand (a handful of families).
//! LENGTH cases: a buffer one element short, at the end of a committed page before a PAGE_NOACCESS page (a fault = HOLE).
//! ALIGNMENT cases: `Plan::execute_*` run an FFTW plan on NEW arrays; FFTW requires them to have the alignment the plan was created for
//! (`FFTW_ESTIMATE` without `FFTW_UNALIGNED`: 16-byte SIMD alignment); a safe `&mut [[f64; 2]]` only guarantees 8. Each alignment case passes
//! an array 8 bytes off the 16-byte boundary and reports (it never asserts: the outcome is measured). One process per test
//! (tools/safe-api-harness.py). Windows only; Linux unmeasured.
#![cfg(windows)]
use aocl_fft::*;
use core::ffi::c_void;

extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
    fn VirtualFree(addr: *mut c_void, size: usize, ty: u32) -> i32;
}
const PAGE: usize = 4096;

/// A buffer of `n` elements ending at a committed page followed by a PAGE_NOACCESS page.
fn at_guard<T: Copy, R>(n: usize, fill: T, f: impl FnOnce(&mut [T]) -> R) -> R {
    assert!(n * core::mem::size_of::<T>() <= PAGE);
    unsafe {
        let base = VirtualAlloc(core::ptr::null_mut(), 2 * PAGE, 0x3000, 0x04);
        assert!(!base.is_null());
        let mut old = 0u32;
        assert_ne!(VirtualProtect((base as *mut u8).add(PAGE) as *mut c_void, PAGE, 0x01, &mut old), 0);
        let start = (base as *mut u8).add(PAGE - n * core::mem::size_of::<T>()) as *mut T;
        for i in 0..n {
            start.add(i).write(fill);
        }
        let r = f(core::slice::from_raw_parts_mut(start, n));
        VirtualFree(base, 0, 0x8000);
        r
    }
}

/// A buffer of `n` elements whose address is `off` bytes past a page boundary (inside committed memory: no guard).
fn at_offset<T: Copy, R>(n: usize, off: usize, fill: T, f: impl FnOnce(&mut [T]) -> R) -> R {
    unsafe {
        let base = VirtualAlloc(core::ptr::null_mut(), 2 * PAGE, 0x3000, 0x04);
        assert!(!base.is_null());
        let start = (base as *mut u8).add(off) as *mut T;
        for i in 0..n {
            start.add(i).write(fill);
        }
        let r = f(core::slice::from_raw_parts_mut(start, n));
        VirtualFree(base, 0, 0x8000);
        r
    }
}

fn report(name: &str, r: Result<()>) {
    println!("RESULT {name} {}", if r.is_err() { "refused" } else { "accepted" });
}

macro_rules! family {
    ($m:ident, $F:ty, $plan:ident, dft2 = $dft2:ident, dft3 = $dft3:ident, r2c = $r2c:ident, c2r = $c2r:ident) => {
        mod $m {
            use super::*;
            const Z: [$F; 2] = [0.0, 0.0];
            #[test]
            fn dft_2d_control() {
                let mut d = vec![Z; 12];
                $dft2(Direction::Forward, 3, 4, &mut d).unwrap();
            }
            #[test]
            fn dft_2d_short_data() {
                report("dft_2d_short_data", at_guard(11, Z, |d| $dft2(Direction::Forward, 3, 4, d)));
            }
            #[test]
            fn dft_3d_control() {
                let mut d = vec![Z; 24];
                $dft3(Direction::Forward, 2, 3, 4, &mut d).unwrap();
            }
            #[test]
            fn dft_3d_short_data() {
                report("dft_3d_short_data", at_guard(23, Z, |d| $dft3(Direction::Forward, 2, 3, 4, d)));
            }
            #[test]
            fn r2c_control() {
                let (mut i, mut o) = (vec![1.0 as $F; 8], vec![Z; 5]);
                $r2c(&mut i, &mut o).unwrap();
            }
            #[test]
            fn r2c_short_output() {
                let mut i = vec![1.0 as $F; 8];
                report("r2c_short_output", at_guard(4, Z, |o| $r2c(&mut i, o)));
            }
            #[test]
            fn c2r_control() {
                let (mut i, mut o) = (vec![Z; 5], vec![0.0 as $F; 8]);
                $c2r(8, &mut i, &mut o).unwrap();
            }
            #[test]
            fn c2r_short_input() {
                let mut o = vec![0.0 as $F; 8];
                report("c2r_short_input", at_guard(4, Z, |i| $c2r(8, i, &mut o)));
            }
            #[test]
            fn c2r_short_output() {
                let mut i = vec![Z; 5];
                report("c2r_short_output", at_guard(7, 0.0 as $F, |o| $c2r(8, &mut i, o)));
            }
            // plan executors
            #[test]
            fn plan_dft_control() {
                let p = $plan::dft_1d(8, Direction::Forward).unwrap();
                let (mut i, mut o) = (vec![Z; 8], vec![Z; 8]);
                p.execute_dft(&mut i, &mut o).unwrap();
            }
            #[test]
            fn plan_dft_short_input() {
                let p = $plan::dft_1d(8, Direction::Forward).unwrap();
                let mut o = vec![Z; 8];
                report("plan_dft_short_input", at_guard(7, Z, |i| p.execute_dft(i, &mut o)));
            }
            #[test]
            fn plan_dft_short_output() {
                let p = $plan::dft_1d(8, Direction::Forward).unwrap();
                let mut i = vec![Z; 8];
                report("plan_dft_short_output", at_guard(7, Z, |o| p.execute_dft(&mut i, o)));
            }
            #[test]
            fn plan_r2c_short_output() {
                let p = $plan::r2c_1d(8).unwrap();
                let mut i = vec![1.0 as $F; 8];
                report("plan_r2c_short_output", at_guard(4, Z, |o| p.execute_r2c(&mut i, o)));
            }
            #[test]
            fn plan_r2c_short_input() {
                let p = $plan::r2c_1d(8).unwrap();
                let mut o = vec![Z; 5];
                report("plan_r2c_short_input", at_guard(7, 1.0 as $F, |i| p.execute_r2c(i, &mut o)));
            }
            #[test]
            fn plan_c2r_short_input() {
                let p = $plan::c2r_1d(8).unwrap();
                let mut o = vec![0.0 as $F; 8];
                report("plan_c2r_short_input", at_guard(4, Z, |i| p.execute_c2r(i, &mut o)));
            }
            #[test]
            fn plan_c2r_short_output() {
                let p = $plan::c2r_1d(8).unwrap();
                let mut i = vec![Z; 5];
                report("plan_c2r_short_output", at_guard(7, 0.0 as $F, |o| p.execute_c2r(&mut i, o)));
            }
            // alignment: the plan was made for aligned scratch; run it on arrays 8 bytes off the 16-byte boundary
            #[test]
            fn plan_dft_misaligned_input() {
                let p = $plan::dft_1d(8, Direction::Forward).unwrap();
                let mut o = vec![Z; 8];
                report("plan_dft_misaligned_input", at_offset(8, 8, Z, |i| p.execute_dft(i, &mut o)));
            }
            #[test]
            fn plan_dft_misaligned_output() {
                let p = $plan::dft_1d(8, Direction::Forward).unwrap();
                let mut i = vec![Z; 8];
                report("plan_dft_misaligned_output", at_offset(8, 8, Z, |o| p.execute_dft(&mut i, o)));
            }
            #[test]
            fn plan_r2c_misaligned_input() {
                let p = $plan::r2c_1d(64).unwrap();
                let mut o = vec![Z; 33];
                report("plan_r2c_misaligned_input", at_offset(64, 8, 1.0 as $F, |i| p.execute_r2c(i, &mut o)));
            }
            #[test]
            fn plan_c2r_misaligned_output() {
                let p = $plan::c2r_1d(64).unwrap();
                let mut i = vec![Z; 33];
                report("plan_c2r_misaligned_output", at_offset(64, 8, 0.0 as $F, |o| p.execute_c2r(&mut i, o)));
            }
        }
    };
}

family!(f64s, f64, Plan, dft2 = dft_2d_inplace, dft3 = dft_3d_inplace, r2c = r2c_1d, c2r = c2r_1d);
family!(f32s, f32, PlanF32, dft2 = dft_2d_inplace_f32, dft3 = dft_3d_inplace_f32, r2c = r2c_1d_f32, c2r = c2r_1d_f32);
