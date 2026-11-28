//! Guard-page harness for the multivariate generators of aocl-rng (`multinormal`, `multi_students_t`, `multinomial`). Hand-written. Each
//! case passes one buffer one element short of what the library reads or writes (`x` is column-major: sample i of column j at
//! `x[i + j * ldx]`, so a call writes `(cols - 1) * ldx + n` elements and reads `(m - 1) * ldc + m` of the covariance), at the end of a
//! committed page before a PAGE_NOACCESS page: a fault = HOLE. One process per test (tools/safe-api-harness.py); controls assert success.
//! Windows only; Linux unmeasured.
#![cfg(windows)]
use aocl_rng::{BaseGenerator, Rng, RngInt};
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
fn rng() -> Rng {
    Rng::new(BaseGenerator::MersenneTwister, 7).unwrap()
}
/// A case that FAULTED before the fix: asserts `InvalidArgument`.
macro_rules! refuse {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            let r = $body;
            assert!(
                matches!(r, Err(aocl_error::Error::InvalidArgument(_))),
                "{}: {:?}",
                stringify!($name),
                r.err()
            );
        }
    };
}
const MU: [f64; 3] = [0.0, 1.0, 2.0];
const C: [f64; 9] = [2.0, 0.1, 0.1, 0.1, 2.0, 0.1, 0.1, 0.1, 2.0]; // 3 x 3, ldc = 3
const P: [f64; 3] = [0.2, 0.3, 0.5];
const N: usize = 4; // samples
#[test]
fn multivariate_controls_control() {
    let (mut mu, mut c) = (MU, C);
    let mut out = vec![0.0f64; N * 3];
    rng()
        .multinormal(3, &mut mu, &mut c, 3, &mut out, N as RngInt)
        .unwrap();
    rng()
        .multi_students_t(3, 8, &mut mu, &mut c, 3, &mut out, N as RngInt)
        .unwrap();
    let mut p = P;
    let mut o = vec![0 as RngInt; N * 3];
    rng()
        .multinomial(10, &mut p, 3, &mut o, N as RngInt)
        .unwrap();
}
refuse!(multinormal_short_mu, {
    let mut c = C;
    let mut out = vec![0.0f64; N * 3];
    at_guard(&MU, 2, |mu| {
        rng().multinormal(3, mu, &mut c, 3, &mut out, N as RngInt)
    })
});
refuse!(multinormal_short_c, {
    let mut mu = MU;
    let mut out = vec![0.0f64; N * 3];
    at_guard(&C, 8, |c| {
        rng().multinormal(3, &mut mu, c, 3, &mut out, N as RngInt)
    })
});
refuse!(multinormal_oversized_ldc, {
    let mut mu = MU;
    let mut out = vec![0.0f64; N * 3];
    at_guard(&C, 9, |c| {
        rng().multinormal(3, &mut mu, c, 4, &mut out, N as RngInt)
    })
});
refuse!(multinormal_oversized_ldx, {
    let mut mu = MU;
    let mut c = C;
    at_guard(&vec![0.0f64; N * 3], N * 3, |out| {
        rng().multinormal(3, &mut mu, &mut c, 3, out, (N + 2) as RngInt)
    })
});
refuse!(multi_students_t_short_mu, {
    let mut c = C;
    let mut out = vec![0.0f64; N * 3];
    at_guard(&MU, 2, |mu| {
        rng().multi_students_t(3, 8, mu, &mut c, 3, &mut out, N as RngInt)
    })
});
refuse!(multi_students_t_short_c, {
    let mut mu = MU;
    let mut out = vec![0.0f64; N * 3];
    at_guard(&C, 8, |c| {
        rng().multi_students_t(3, 8, &mut mu, c, 3, &mut out, N as RngInt)
    })
});
refuse!(multi_students_t_oversized_ldc, {
    let mut mu = MU;
    let mut out = vec![0.0f64; N * 3];
    at_guard(&C, 9, |c| {
        rng().multi_students_t(3, 8, &mut mu, c, 4, &mut out, N as RngInt)
    })
});
refuse!(multi_students_t_oversized_ldx, {
    let mut mu = MU;
    let mut c = C;
    at_guard(&vec![0.0f64; N * 3], N * 3, |out| {
        rng().multi_students_t(3, 8, &mut mu, &mut c, 3, out, (N + 2) as RngInt)
    })
});
refuse!(multinomial_short_p, {
    let mut o = vec![0 as RngInt; N * 3];
    at_guard(&P, 2, |p| rng().multinomial(10, p, 3, &mut o, N as RngInt))
});
refuse!(multinomial_oversized_ldx, {
    let mut p = P;
    at_guard(&vec![0 as RngInt; N * 3], N * 3, |o| {
        rng().multinomial(10, &mut p, 3, o, (N + 2) as RngInt)
    })
});
