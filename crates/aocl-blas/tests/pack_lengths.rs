//! Guard-page harness for the aocl-blas packed / compute / 3m GEMM free functions (`?gemm_pack`, `?gemm_compute`, `cgemm3m`, `zgemm3m`), hand-written.
//! Control = exact sizes (the pack destination is `?gemm_pack_get_size` BYTES); each short case passes ONE buffer one element short at the end of a
//! committed page before a PAGE_NOACCESS page (a fault = HOLE). One process per test (tools/safe-api-harness.py). Windows only; Linux unmeasured.
#![cfg(windows)]
use aocl_blas::*;
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
const M: usize = 6;
const N: usize = 5;
const K: usize = 4;
const NO: i32 = 111; // CblasNoTrans, an unpacked operand for ?gemm_compute
/// 14 of these 20 cases ACCESS-VIOLATED before the checks (and the functions returned `()`): every short case must now be refused.
macro_rules! case {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            let r = $body;
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

fn pk_f64() -> usize {
    dgemm_pack_get_size(PackId::A, M, N, K).div_ceil(core::mem::size_of::<f64>())
}
control!(pack_f64_control, {
    let src = vec![1.0f64; M * K];
    let mut d = vec![0.0f64; pk_f64()];
    dgemm_pack(
        Layout::RowMajor,
        PackId::A,
        Trans::No,
        M,
        N,
        K,
        1.0,
        &src,
        K,
        &mut d,
    )
});
case!(pack_f64_short_src, {
    let mut d = vec![0.0f64; pk_f64()];
    at_guard(&[1.0f64; M * K], M * K - 1, |src| {
        dgemm_pack(
            Layout::RowMajor,
            PackId::A,
            Trans::No,
            M,
            N,
            K,
            1.0,
            src,
            K,
            &mut d,
        )
    })
});
case!(pack_f64_short_dest, {
    let src = vec![1.0f64; M * K];
    at_guard(&vec![0.0f64; pk_f64()], pk_f64() - 1, |d| {
        dgemm_pack(
            Layout::RowMajor,
            PackId::A,
            Trans::No,
            M,
            N,
            K,
            1.0,
            &src,
            K,
            d,
        )
    })
});
control!(compute_f64_control, {
    let (a, b, mut c) = (
        vec![1.0f64; M * K],
        vec![1.0f64; K * N],
        vec![0.0f64; M * N],
    );
    dgemm_compute(
        Layout::RowMajor,
        NO,
        NO,
        M,
        N,
        K,
        &a,
        K,
        &b,
        N,
        0.0,
        &mut c,
        N,
    )
});
case!(compute_f64_short_a, {
    let (b, mut c) = (vec![1.0f64; K * N], vec![0.0f64; M * N]);
    at_guard(&[1.0f64; M * K], M * K - 1, |a| {
        dgemm_compute(
            Layout::RowMajor,
            NO,
            NO,
            M,
            N,
            K,
            a,
            K,
            &b,
            N,
            0.0,
            &mut c,
            N,
        )
    })
});
case!(compute_f64_short_b, {
    let (a, mut c) = (vec![1.0f64; M * K], vec![0.0f64; M * N]);
    at_guard(&[1.0f64; K * N], K * N - 1, |b| {
        dgemm_compute(
            Layout::RowMajor,
            NO,
            NO,
            M,
            N,
            K,
            &a,
            K,
            b,
            N,
            0.0,
            &mut c,
            N,
        )
    })
});
case!(compute_f64_short_c, {
    let (a, b) = (vec![1.0f64; M * K], vec![1.0f64; K * N]);
    at_guard(&[0.0f64; M * N], M * N - 1, |c| {
        dgemm_compute(Layout::RowMajor, NO, NO, M, N, K, &a, K, &b, N, 0.0, c, N)
    })
});

fn pk_f32() -> usize {
    sgemm_pack_get_size(PackId::A, M, N, K).div_ceil(core::mem::size_of::<f32>())
}
control!(pack_f32_control, {
    let src = vec![1.0f32; M * K];
    let mut d = vec![0.0f32; pk_f32()];
    sgemm_pack(
        Layout::RowMajor,
        PackId::A,
        Trans::No,
        M,
        N,
        K,
        1.0,
        &src,
        K,
        &mut d,
    )
});
case!(pack_f32_short_src, {
    let mut d = vec![0.0f32; pk_f32()];
    at_guard(&[1.0f32; M * K], M * K - 1, |src| {
        sgemm_pack(
            Layout::RowMajor,
            PackId::A,
            Trans::No,
            M,
            N,
            K,
            1.0,
            src,
            K,
            &mut d,
        )
    })
});
case!(pack_f32_short_dest, {
    let src = vec![1.0f32; M * K];
    at_guard(&vec![0.0f32; pk_f32()], pk_f32() - 1, |d| {
        sgemm_pack(
            Layout::RowMajor,
            PackId::A,
            Trans::No,
            M,
            N,
            K,
            1.0,
            &src,
            K,
            d,
        )
    })
});
control!(compute_f32_control, {
    let (a, b, mut c) = (
        vec![1.0f32; M * K],
        vec![1.0f32; K * N],
        vec![0.0f32; M * N],
    );
    sgemm_compute(
        Layout::RowMajor,
        NO,
        NO,
        M,
        N,
        K,
        &a,
        K,
        &b,
        N,
        0.0,
        &mut c,
        N,
    )
});
case!(compute_f32_short_a, {
    let (b, mut c) = (vec![1.0f32; K * N], vec![0.0f32; M * N]);
    at_guard(&[1.0f32; M * K], M * K - 1, |a| {
        sgemm_compute(
            Layout::RowMajor,
            NO,
            NO,
            M,
            N,
            K,
            a,
            K,
            &b,
            N,
            0.0,
            &mut c,
            N,
        )
    })
});
case!(compute_f32_short_b, {
    let (a, mut c) = (vec![1.0f32; M * K], vec![0.0f32; M * N]);
    at_guard(&[1.0f32; K * N], K * N - 1, |b| {
        sgemm_compute(
            Layout::RowMajor,
            NO,
            NO,
            M,
            N,
            K,
            &a,
            K,
            b,
            N,
            0.0,
            &mut c,
            N,
        )
    })
});
case!(compute_f32_short_c, {
    let (a, b) = (vec![1.0f32; M * K], vec![1.0f32; K * N]);
    at_guard(&[0.0f32; M * N], M * N - 1, |c| {
        sgemm_compute(Layout::RowMajor, NO, NO, M, N, K, &a, K, &b, N, 0.0, c, N)
    })
});

control!(g3m_c64_control, {
    let (a, b, mut c) = (
        vec![Complex64::new(1.0, 0.0); M * K],
        vec![Complex64::new(1.0, 0.0); K * N],
        vec![Complex64::new(0.0, 0.0); M * N],
    );
    zgemm3m(
        Layout::RowMajor,
        Trans::No,
        Trans::No,
        M,
        N,
        K,
        Complex64::new(1.0, 0.0),
        &a,
        K,
        &b,
        N,
        Complex64::new(0.0, 0.0),
        &mut c,
        N,
    )
});
case!(g3m_c64_short_a, {
    let (b, mut c) = (
        vec![Complex64::new(1.0, 0.0); K * N],
        vec![Complex64::new(0.0, 0.0); M * N],
    );
    at_guard(&[Complex64::new(1.0, 0.0); M * K], M * K - 1, |a| {
        zgemm3m(
            Layout::RowMajor,
            Trans::No,
            Trans::No,
            M,
            N,
            K,
            Complex64::new(1.0, 0.0),
            a,
            K,
            &b,
            N,
            Complex64::new(0.0, 0.0),
            &mut c,
            N,
        )
    })
});
case!(g3m_c64_short_b, {
    let (a, mut c) = (
        vec![Complex64::new(1.0, 0.0); M * K],
        vec![Complex64::new(0.0, 0.0); M * N],
    );
    at_guard(&[Complex64::new(1.0, 0.0); K * N], K * N - 1, |b| {
        zgemm3m(
            Layout::RowMajor,
            Trans::No,
            Trans::No,
            M,
            N,
            K,
            Complex64::new(1.0, 0.0),
            &a,
            K,
            b,
            N,
            Complex64::new(0.0, 0.0),
            &mut c,
            N,
        )
    })
});
case!(g3m_c64_short_c, {
    let (a, b) = (
        vec![Complex64::new(1.0, 0.0); M * K],
        vec![Complex64::new(1.0, 0.0); K * N],
    );
    at_guard(&[Complex64::new(0.0, 0.0); M * N], M * N - 1, |c| {
        zgemm3m(
            Layout::RowMajor,
            Trans::No,
            Trans::No,
            M,
            N,
            K,
            Complex64::new(1.0, 0.0),
            &a,
            K,
            &b,
            N,
            Complex64::new(0.0, 0.0),
            c,
            N,
        )
    })
});

control!(g3m_c32_control, {
    let (a, b, mut c) = (
        vec![Complex32::new(1.0, 0.0); M * K],
        vec![Complex32::new(1.0, 0.0); K * N],
        vec![Complex32::new(0.0, 0.0); M * N],
    );
    cgemm3m(
        Layout::RowMajor,
        Trans::No,
        Trans::No,
        M,
        N,
        K,
        Complex32::new(1.0, 0.0),
        &a,
        K,
        &b,
        N,
        Complex32::new(0.0, 0.0),
        &mut c,
        N,
    )
});
case!(g3m_c32_short_a, {
    let (b, mut c) = (
        vec![Complex32::new(1.0, 0.0); K * N],
        vec![Complex32::new(0.0, 0.0); M * N],
    );
    at_guard(&[Complex32::new(1.0, 0.0); M * K], M * K - 1, |a| {
        cgemm3m(
            Layout::RowMajor,
            Trans::No,
            Trans::No,
            M,
            N,
            K,
            Complex32::new(1.0, 0.0),
            a,
            K,
            &b,
            N,
            Complex32::new(0.0, 0.0),
            &mut c,
            N,
        )
    })
});
case!(g3m_c32_short_b, {
    let (a, mut c) = (
        vec![Complex32::new(1.0, 0.0); M * K],
        vec![Complex32::new(0.0, 0.0); M * N],
    );
    at_guard(&[Complex32::new(1.0, 0.0); K * N], K * N - 1, |b| {
        cgemm3m(
            Layout::RowMajor,
            Trans::No,
            Trans::No,
            M,
            N,
            K,
            Complex32::new(1.0, 0.0),
            &a,
            K,
            b,
            N,
            Complex32::new(0.0, 0.0),
            &mut c,
            N,
        )
    })
});
case!(g3m_c32_short_c, {
    let (a, b) = (
        vec![Complex32::new(1.0, 0.0); M * K],
        vec![Complex32::new(1.0, 0.0); K * N],
    );
    at_guard(&[Complex32::new(0.0, 0.0); M * N], M * N - 1, |c| {
        cgemm3m(
            Layout::RowMajor,
            Trans::No,
            Trans::No,
            M,
            N,
            K,
            Complex32::new(1.0, 0.0),
            &a,
            K,
            &b,
            N,
            Complex32::new(0.0, 0.0),
            c,
            N,
        )
    })
});
