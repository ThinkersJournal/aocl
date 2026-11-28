//! MEASUREMENT of the `unsafe` initialisers: does the library read more parameter words than `n` says? Each test places its
//! parameters at the END of a committed page followed by a PAGE_NOACCESS page: a read past `n` is an access violation
//! (0xc0000005) and kills the process, so `run_wrapper_tests.py` runs each test by exact name in its own process and records
//! the exit code. A test that returns normally means "no read past n observed for this n". Windows only; PM-run; the lane
//! does not run the library. Compiled only with OPENRNG_LIB_DIR.
#![cfg(all(openrng_linked, windows))]
use aocl_openrng::*;
use core::ffi::c_void;

extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
    fn VirtualFree(addr: *mut c_void, size: usize, ty: u32) -> i32;
}
const PAGE: usize = 4096;

/// Run `f` with a slice of `n` T at the end of a readable page, the next page inaccessible.
fn at_guard<T: Copy + Default, R>(n: usize, f: impl FnOnce(&[T]) -> R) -> R {
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
            start.add(i).write(T::default());
        }
        let r = f(core::slice::from_raw_parts(start, n));
        VirtualFree(base, 0, 0x8000);
        r
    }
}

macro_rules! guard_params {
    ($($name:ident: $brng:expr, $n:expr;)*) => {$(
        #[test]
        fn $name() {
            let r = at_guard::<u32, _>($n, |p| unsafe { Stream::from_params($brng, p) }.map(|_| ()));
            println!("GUARD from_params({:?}, n={}) returned {:?}", $brng, $n, r);
        }
    )*};
}

guard_params! {
    philox_n1: Brng::Philox4x32x10, 1;
    philox_n2: Brng::Philox4x32x10, 2;
    philox_n3: Brng::Philox4x32x10, 3;
    mt19937_n1: Brng::Mt19937, 1;
    mt19937_n2: Brng::Mt19937, 2;
    mrg32k3a_n1: Brng::Mrg32k3a, 1;
    mrg32k3a_n3: Brng::Mrg32k3a, 3;
    mcg31_n1: Brng::Mcg31, 1;
    r250_n1: Brng::R250, 1;
    sfmt_n1: Brng::Sfmt19937, 1;
    ars5_n1: Brng::Ars5, 1;
    ars5_n2: Brng::Ars5, 2;
    sobol_n1: Brng::Sobol, 1;
    niederr_n1: Brng::Niederr, 1;
}

macro_rules! guard_skip {
    ($($name:ident: $n:expr;)*) => {$(
        #[test]
        fn $name() {
            let r = at_guard::<u64, _>($n, |p| {
                let mut s = Stream::new(Brng::Philox4x32x10, 1).unwrap();
                unsafe { s.skip_ahead_ex(p) }
            });
            println!("GUARD skip_ahead_ex(Philox, n={}) returned {:?}", $n, r);
        }
    )*};
}

guard_skip! {
    skip_ex_philox_n1: 1;
    skip_ex_philox_n2: 2;
}
